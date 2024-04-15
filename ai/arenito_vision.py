# pyright: strict

from __future__ import annotations
import cv2
import math
import numpy as np
import numpy.typing as ntp
from argparse import Namespace
from typing import NamedTuple, Sequence
from cv2.typing import MatLike, RotatedRect
from arenito_com import AIMode
from logging import Logger

WHITE = (255, 255, 255)
BLACK = (0, 0, 0)
BLUE = (255, 0, 0)
GREEN = (0, 255, 0)
RED = (0, 0, 255)
ORANGE = (3, 102, 252)

class Point(NamedTuple):
    """
    A basic point implementation.
    """

    x: int
    y: int

class Rect(NamedTuple):
    """
    A basic rect implementation.
    """

    a: Point # bottom left
    b: Point # top right

    def point_inside(self, point: Point) -> bool:
        x = self.a.x <= point.x <= self.b.x
        y = self.a.y <= point.y <= self.b.y
        return x and y

class Detection:
    """
    Stores detection data.
    """

    def __init__(self, rect: RotatedRect, contour: MatLike):
        self.rect = rect
        self.box = np.int0(cv2.boxPoints(rect)) # pyright: ignore
        self.center = Point(
            sum(self.box[:,0]) // 4, # pyright: ignore
            sum(self.box[:,1]) // 4  # pyright: ignore
        )
        self.contour = contour

    @staticmethod
    def from_point(point: Point) -> Detection:
        A = np.array((5, 5))
        B = np.array((-5, 5))
        D = np.array((5, -5))
        C = np.array((-5, -5))
        BASE = np.array([*point])

        cnt = np.array((BASE + A, BASE + B, BASE + C, BASE + D))
        rect = cv2.minAreaRect(cnt)
        return Detection(rect, cnt)

class BlobDetector:
    """
    My blob detector class.
    """

    def __init__(self, params): # pyright: ignore[reportMissingParameterType, reportUnknownParameterType]
        self.detector = cv2.SimpleBlobDetector.create(params)

    @staticmethod
    def can_detector(min_can_area: int) -> BlobDetector:
        """
        Blob detector tuned for cans.
        """

        params = cv2.SimpleBlobDetector.Params()
        params.filterByArea = True
        params.minArea = min_can_area
        params.maxArea = 300000
        params.filterByCircularity = False
        params.filterByConvexity = False
        params.filterByInertia = True
        params.minInertiaRatio = 0.01
        params.maxInertiaRatio = 0.7

        return BlobDetector(params)

    def detect(self, image_hsv: MatLike) -> Sequence[cv2.KeyPoint]:
        """
        Runs image through blob detector.
        """

        return self.detector.detect(image_hsv)

    def points(self, image_hsv: MatLike) -> list[Point]:
        """
        Runs image through blob detector. Returns list of Point instead of cv2.KeyPoint.
        """

        keypoints = self.detect(image_hsv)
        return [
            Point(*map(int, k.pt))
            for k in keypoints
        ]

ColorF = tuple[ntp.NDArray[np.int8], ntp.NDArray[np.int8]]
class ColorFilter:
    """
    Stores color filter data.
    delete mongodb, flutter, sass
    """

    BLUE = (
        np.array([39, 76, 110]),   # lower
        np.array([175, 255, 255]), # upper
        # np.array([75, 160, 88]),   # lower
        # np.array([175, 255, 255]), # upper
        # np.array([57, 76, 77]),   # lower
        # np.array([118, 255, 210]), # upper
    )
    RED = (
        np.array([0, 176, 0]),
        np.array([78, 255, 255]),
        # np.array([0, 107, 44]),
        # np.array([179, 255, 144]),
    )
    BLACK = (
        np.array([0, 0, 69]),      # lower
        np.array([179, 255, 255]), # upper
    )

    @staticmethod
    def filter(img_hsv: MatLike, color: ColorF) -> MatLike:
        """
        Applies color filter to hsv_img and returns mask.
        """

        lower, upper = color
        return cv2.inRange(img_hsv, lower, upper)

class ArenitoVision:
    """
    This is where every vision-related operation will be handled.
    """

    RESOLUTIONS = {
        AIMode.Simulation : (512, 512),
        AIMode.Jetson : (512, 512),
    }

    def __init__(self, mode: AIMode, args: Namespace, logger: Logger):
        if mode == AIMode.Simulation or mode == AIMode.Jetson:
            res = ArenitoVision.RESOLUTIONS[mode]
        else:
            raise Exception(f'Unsupported mode mode {mode}')

        match args.algorithm:
            case 'blob-detector':
                self.can_detection_function = self.blob_detector_method
            case 'min-rect':
                self.can_detection_function = self.min_rect_method
            case other:
                raise Exception(f'Unsupported algorithm {other}')

        self.logger = logger
        self.img_counter = 0
        self.save_images: bool = args.save_images

        self.res_x, self.res_y = res

        # Bottom center of the image
        # +-----------------------+
        # |                       |
        # |                       |
        # |                       |
        # +-----------X-----------+
        self.bottom_center = Point(self.res_x // 2, self.res_y)

        # How close to the water is the robot allowed to be.
        # When no cans are found, move forward until running into water, then rotate.
        # The robot determines that it's run into water when the point directly forward
        # is reachable. This dot is called `blue_r_dot`. If `blue_r_dot` is reachable, the robot
        # can continue forward. Otherwise, the robot is by the edge of the traversable area
        # and, if it goes forward, it'll go out into the "water".
        # +------------------------+
        # |                        |
        # |            X           |
        # |           ###          |
        # |           ###          |
        # +------------------------+
        self.water_dist_from_center = 100
        self.blue_r_dot = Point(self.res_x // 2, self.res_y // 2 + self.water_dist_from_center)
        # Theres also a `dump_r_dot`, for the dump...
        self.dump_dist_from_center = 20
        self.dump_r_dot = Point(self.res_x // 2, self.res_y // 2 + self.dump_dist_from_center)

        # Area limits where a detection is considered to be centered.
        # +------------------------+
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # +------------------------+
        margin_x = int(self.res_x * 0.2)
        self.can_threshold_x = (
            self.res_x // 2 - margin_x, # min
            self.res_x // 2 + margin_x  # max
        )
        margin_x = int(self.res_x * 0.04)
        self.deposit_threshold_x = (
            self.res_x // 2 - margin_x, # min
            self.res_x // 2 + margin_x  # max
        )

        # When finding out if a point is reachable, counts how many blue pixels
        # there are between the robot and that point.
        # This is the minimum ammount of blue pixels necessary between the robot
        # and any given point for it to be considered `unreachable`.
        self.min_px_water = 50

        # This limits the bottom collision-with-blue area
        # +------------------------+
        # |                        |
        # |                        |
        # |- - - - - - - - - - - - | <- This line
        # +------------------------+
        # previously 380 - 20, where 380 = res_y
        self.bottom_line_y = int(self.res_y * 0.9473)

        # This limits the vertical collision-with-blue area
        # +------------------------+
        # |                        |
        # |           __           |
        # |          |  |          |
        # |          |  |          |
        # +------------------------+
        # previously 140
        self.vertical_line_thickness = int(self.res_x * 0.21875)

        # Combining both bottom_line and vertical_line gives us the mask
        # of the collision-with-blue area.
        # +------------------------+
        # |                        |
        # |           __           |
        # |          |  |          |
        # |----------|--|----------|
        # +------------------------+

        # Minimum size for a rect to be considered a can
        self.min_can_area = 100
        self.min_dump_area = 600
        self.can_blob_detector = BlobDetector.can_detector(self.min_can_area)

        # Can critical region: The area with which will decide if a can was or not grabbed
        # +------------------------+
        # |                        |
        # |                        |
        # |                        |
        # |       ##########       |
        # +-------##########-------+
        # Arenito will remember if a can is visible within this area, the moment it stopps
        # being visible, that can most probably was grabbed.
        self.can_critical_region = Rect(
            Point(int(self.res_x * 0.2), int(self.res_y * 0.83)),
            Point(int(self.res_x * 0.8), int(self.res_y)),
        )
        # Same for deposit's critial region
        self.deposit_critical_region = Rect(
            Point(int(self.res_x * 0.23), int(self.res_y * 0.6)),
            Point(int(self.res_x * 0.77), int(self.res_y)),
        )

    def add_text(self, img: MatLike, text: str, pos: Point):
        """
        Draws a text with the default configuration in the specified position.
        """

        cv2.putText(img, text, pos, cv2.QT_FONT_NORMAL, 0.55, WHITE, 1, cv2.LINE_AA)

    def add_markings(
        self,
        det_img: MatLike,
        detections: list[Detection],
        state: str,
        can_counter: int,
        cicr: bool,
        dump: None | Detection,
        clock: str,
    ):
        """
        Adds visual markings to image to help visualize decisions.
        """

        can_counter_str = f'Cans: {can_counter}'
        if cicr:
            can_counter_str += ' (In critical region)'

        self.add_text(det_img, can_counter_str, Point(10, 35))
        self.add_text(det_img, state, Point(10, 55))

        t = self.vertical_line_thickness // 2
        a1 = Point(self.bottom_center.x - t, self.bottom_center.y)
        b1 = Point(a1.x, self.blue_r_dot.y)
        a2 = Point(self.bottom_center.x + t, self.bottom_center.y)
        b2 = Point(a2.x, self.blue_r_dot.y)

        cv2.line(det_img, a1, b1, BLUE)
        cv2.line(det_img, a2, b2, BLUE)
        cv2.ellipse(det_img, self.blue_r_dot, (t, t), 0.0, 180.0, 360.0, BLUE, thickness=1)

        a1 = Point(self.bottom_center.x - t, self.bottom_center.y)
        b1 = Point(a1.x, self.dump_r_dot.y)
        a2 = Point(self.bottom_center.x + t, self.bottom_center.y)
        b2 = Point(a2.x, self.dump_r_dot.y)

        cv2.line(det_img, a1, b1, RED)
        cv2.line(det_img, a2, b2, RED)
        cv2.ellipse(det_img, self.dump_r_dot, (t, t), 0.0, 180.0, 360.0, RED, thickness=1)

        cv2.rectangle(
            det_img,
            self.can_critical_region.a,
            self.can_critical_region.b,
            BLACK,
            1
        )

        cv2.rectangle(
            det_img,
            self.deposit_critical_region.a,
            self.deposit_critical_region.b,
            ORANGE,
            1
        )

        cv2.line(
            det_img,
            (0, self.bottom_line_y),
            (self.res_x, self.bottom_line_y),
            WHITE,
            thickness=1
        )
        cv2.line(
            det_img,
            (self.can_threshold_x[0], 0),
            (self.can_threshold_x[0], self.res_y),
            WHITE,
            thickness=1,
        )
        cv2.line(
            det_img,
            (self.can_threshold_x[1], 0),
            (self.can_threshold_x[1], self.res_y),
            WHITE,
            thickness=1,
        )

        for det in detections:
            cv2.circle(det_img, det.center, 10, WHITE, 10)
            cv2.drawContours(det_img, [det.contour], -1, GREEN, 1, cv2.LINE_AA) # pyright: ignore
            cv2.drawContours(det_img, [det.box], -1, RED, 1, cv2.LINE_AA) # pyright: ignore

        if detections:
            cv2.circle(det_img, detections[0].center, 10, BLUE, 10)

        if dump:
            cv2.circle(det_img, dump.center, 10, ORANGE, 10)
            cv2.drawContours(det_img, [dump.box], -1, RED, 1, cv2.LINE_AA) # pyright: ignore

        self.add_text(det_img, f'Time: {clock}', Point(10, 75))

        if self.save_images:
            img_filename = f'img/markings_{self.img_counter}.jpg'
            cv2.imwrite(img_filename, det_img)
            self.logger.info(f'Saved image "{img_filename}"')

    def dist_from_center(self, det: Point) -> float:
        """
        Calculates the distance from `self.bottom_center` to `det`.
        """

        x1, y1 = self.bottom_center
        x2, y2 = det

        return math.sqrt((x2 - x1)**2 + (y2 - y1)**2)

    def reachable_old(self, img_hsv: MatLike, det: Point) -> bool:
        """
        Determines if a detection is reachable. Returns true if possible, otherwise false.
        """

        mask_blue = ColorFilter.filter(img_hsv, ColorFilter.BLUE)
        mask_red = ColorFilter.filter(img_hsv, ColorFilter.RED)

        mask = cv2.bitwise_or(mask_blue, mask_red)

        line = np.zeros(shape=mask.shape, dtype=np.uint8)
        cv2.line(line, self.bottom_center, det, WHITE, thickness=self.vertical_line_thickness)
        cv2.rectangle(line, (0, self.bottom_line_y), (self.res_x, self.res_y), WHITE, thickness=-1)

        cross = cv2.bitwise_and(mask, line)
        white_px = np.count_nonzero(cross)

        if self.save_images:
            mask_filename = f'img/mask_{self.img_counter}.jpg'
            mask_red_filename = f'img/mask_red_{self.img_counter}.jpg'
            mask_blue_filename = f'img/mask_blue_{self.img_counter}.jpg'
            reachable_filename = f'img/reachable_{self.img_counter}.jpg'
            cv2.imwrite(mask_filename, mask)
            cv2.imwrite(mask_red_filename, mask_red)
            cv2.imwrite(mask_blue_filename, mask_blue)
            cv2.imwrite(reachable_filename, cross)
            self.logger.info(f'Saved image "{mask_filename}"')
            self.logger.info(f'Saved image "{mask_red_filename}"')
            self.logger.info(f'Saved image "{mask_blue_filename}"')
            self.logger.info(f'Saved image "{reachable_filename}"')

        return white_px < self.min_px_water

    def reachable(
        self,
        img_hsv: MatLike,
        det: Point,
        filter_red: bool = True,
        secondary_det: Point = Point(0, 0),
    ) -> bool:
        """
        Determines if a detection is reachable. Returns true if possible, otherwise false.
        """

        mask_blue = ColorFilter.filter(img_hsv, ColorFilter.BLUE)
        line_blue = np.zeros(shape=mask_blue.shape, dtype=np.uint8)
        cv2.line(line_blue, self.bottom_center, det, WHITE, thickness=self.vertical_line_thickness)
        cv2.rectangle(line_blue, (0, self.bottom_line_y), (self.res_x, self.res_y), WHITE, thickness=-1)

        cross = cv2.bitwise_and(mask_blue, line_blue)
        white_px_blue = np.count_nonzero(cross)

        if self.save_images:
            mask_blue_filename = f'img/mask_blue_{self.img_counter}.jpg'
            reachable_blue_filename = f'img/reachable_blue_{self.img_counter}.jpg'
            cv2.imwrite(mask_blue_filename, mask_blue)
            cv2.imwrite(reachable_blue_filename, cross)
            self.logger.info(f'Saved image "{mask_blue_filename}"')
            self.logger.info(f'Saved image "{reachable_blue_filename}"')

        if filter_red:
            mask_red = ColorFilter.filter(img_hsv, ColorFilter.RED)
            line_red = np.zeros(shape=mask_red.shape, dtype=np.uint8)
            cv2.line(line_red, self.bottom_center, secondary_det, WHITE, thickness=self.vertical_line_thickness)
            cv2.rectangle(line_red, (0, self.bottom_line_y), (self.res_x, self.res_y), WHITE, thickness=-1)

            cross = cv2.bitwise_and(mask_red, line_red)
            white_px_red = np.count_nonzero(cross)

            if self.save_images:
                mask_red_filename = f'img/mask_red_{self.img_counter}.jpg'
                reachable_red_filename = f'img/reachable_red_{self.img_counter}.jpg'
                cv2.imwrite(mask_red_filename, mask_red)
                cv2.imwrite(reachable_red_filename, cross)
                self.logger.info(f'Saved image "{mask_red_filename}"')
                self.logger.info(f'Saved image "{reachable_red_filename}"')

            return (white_px_blue + white_px_red) < self.min_px_water
        else:
            return white_px_blue < self.min_px_water

    def min_rect_method(self, img: MatLike) -> list[Detection]:
        """
        Filters out cans by color and size utilizing cv2.minAreaRect().
        """

        # Without this cans that are on the border are invisible
        gray = cv2.copyMakeBorder(img, 1, 1, 1, 1, cv2.BORDER_CONSTANT, None, WHITE)

        # need better filter
        # gray = cv2.cvtColor(gray, cv2.COLOR_RGB2GRAY)
        # _, mask = cv2.threshold(gray, 50, 255, cv2.RETR_EXTERNAL)

        gray = cv2.cvtColor(gray, cv2.COLOR_BGR2HSV)
        mask = ColorFilter.filter(gray, ColorFilter.BLACK)

        contours, _ = cv2.findContours(
            mask,
            cv2.RETR_TREE,
            cv2.CHAIN_APPROX_NONE
        )

        img_h, img_w, _ = img.shape
        img_h -= 5
        img_w -= 5
        # BGR -> HSV instead of RGB -> HSV because ...?
        img_hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)

        detections: list[Detection] = []
        for cnt in contours:
            rect = cv2.minAreaRect(cnt)
            w, h = rect[1]

            # discard full image contours
            if w >= img_w or h >= img_h:
                continue

            area = w * h
            if area > self.min_can_area:
                det = Detection(rect, cnt)

                # discard really long rectangles
                if w / h < 0.5: continue

                if self.reachable(img_hsv, det.center, secondary_det=det.center):
                    detections.append(det)

        if self.save_images:
            img_filename = f'img/cans_{self.img_counter}.jpg'
            cv2.imwrite(img_filename, mask)
            self.logger.info(f'Saved image "{img_filename}"')

        detections.sort(key=lambda n: self.dist_from_center(n.center), reverse=False)

        return detections

    def blob_detector_method(self, img: MatLike) -> list[Detection]:
        """
        Filters out cans by color and size using cv2's blob detector.
        """

        # Este borde es necesario porque sino no detecta las latas cerca
        # de las esquinas de la imagen
        img = cv2.copyMakeBorder(img, 1, 1, 1, 1, cv2.BORDER_CONSTANT, None, WHITE)

        hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
        black_mask = ColorFilter.filter(hsv, ColorFilter.BLACK)
        detection_points = self.can_blob_detector.points(black_mask)

        return [
            Detection.from_point(point)
            for point in detection_points
            if self.reachable(hsv, point)
        ]

    def find_cans(self, img: MatLike) -> list[Detection]:
        """
        Runs the can detection algorithm.
        """

        return self.can_detection_function(img)

    def blur(self, img: MatLike) -> MatLike:
        """
        Applies a blur filter.
        """

        # img = cv2.bilateralFilter(img, 25, 100, 100)
        # img = cv2.medianBlur(img, 9)
        # this seems to be the best compromise between performance and results
        blurred = cv2.GaussianBlur(img, (51, 51), 0)

        if self.save_images:
            img_filename = f'img/blurred_{self.img_counter}.jpg'
            cv2.imwrite(img_filename, blurred)
            self.logger.info(f'Saved image "{img_filename}"')
            # self.logger.info(f'with brightness mean: {self.get_mean(blurred)}') # pyright: ignore

        return blurred

    def can_in_critical_region(self, detections: list[Detection]) -> bool:
        """
        Returns true when the closest can is in the critical zone.
        """

        if not detections:
            return False

        return self.can_critical_region.point_inside(detections[0].center)

    def detect_dumping_zone(self, blurred_img: MatLike) -> None | Detection:
        """
        Filters out red color and returns a point indicating where it is.
        """

        img_hsv = cv2.cvtColor(blurred_img, cv2.COLOR_BGR2HSV)
        filter_red = ColorFilter.filter(img_hsv, ColorFilter.RED)
        contours, _ = cv2.findContours(filter_red, cv2.RETR_TREE, cv2.CHAIN_APPROX_NONE)

        if not contours: return None

        detections: list[Detection] = []
        for c in contours:
            rect = cv2.minAreaRect(c)
            w, h = rect[1]
            if w * h < self.min_dump_area:
            # return None
                continue

            detections.append(Detection(rect, c))

        if not detections:
            return None

        detections.sort(key=lambda n: self.dist_from_center(n.center), reverse=False)

        self.logger.info(f'Detecting dump')
        if not self.reachable(img_hsv, detections[0].center, filter_red=False):
            return None

        return detections[0]
