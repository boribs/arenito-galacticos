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
        np.array([75, 160, 88]),   # lower
        np.array([175, 255, 255]), # upper
        # np.array([57, 76, 77]),   # lower
        # np.array([118, 255, 210]), # upper
    )
    RED = (
        np.array([0, 176, 0]),
        np.array([78, 255, 255]),
    )
    BLACK = (
        np.array([0, 0, 69]),      # lower
        np.array([175, 255, 255]), # upper
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
        AIMode.Real : (640, 380),
    }

    CAN_CRITICAL_REGIONS = {
        AIMode.Simulation : Rect(
            Point(102, 430),
            Point(410, 512)
        ),
        AIMode.Real : Rect(Point(0, 0), Point(0, 0)),
    }

    DEPOSIT_CRITICAL_REGIONS = {
        AIMode.Simulation : Rect(
            Point(120, 300),
            Point(392, 512)
        ),
        AIMode.Real : Rect(Point(0, 0), Point(0, 0)),
    }

    def __init__(self, mode: AIMode, args: Namespace):
        if mode == AIMode.Simulation or mode == AIMode.Real:
            res = ArenitoVision.RESOLUTIONS[mode]
        else:
            raise Exception(f'No such mode {mode}')

        match args.algorithm:
            case 'blob-detector':
                self.can_detection_function = self.blob_detector_method
            case 'min-rect':
                self.can_detection_function = self.min_rect_method
            case other:
                raise Exception(f'no such algorithm {other}')

        self.res_x, self.res_y = res
        self.margin_x = int(self.res_x * 0.2)

        # Bottom center of the image
        # +------------------------+
        # |                        |
        # |                        |
        # |                        |
        # +------------X-----------+
        self.bottom_center = Point(self.res_x // 2, self.res_y)

        # How close to the water is the robot allowed to be.
        # When no cans are found, move forward until running into water, then rotate.
        # The robot determines that it's run into water when the point directly forward
        # is reachable. This dot is called `r_dot`. If `r_dot` is reachable, the robot
        # can continue forward. Otherwise, the robot is by the edge of the traversable area
        # and, if it goes forward, it'll go out into the "water".
        # +------------------------+
        # |                        |
        # |            X           |
        # |           ###          |
        # |           ###          |
        # +------------------------+
        self.water_dist_from_center = 90
        self.r_dot = Point(self.res_x // 2, self.res_y // 2 + self.water_dist_from_center)

        # Area limits where a detection is considered to be centered.
        # +------------------------+
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # +------------------------+
        self.center_x_min = self.res_x // 2 - self.margin_x
        self.center_x_max = self.res_x // 2 + self.margin_x

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
        self.min_can_area = 700
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
        self.can_critical_region = ArenitoVision.CAN_CRITICAL_REGIONS[mode]
        # Same for deposit's critial region
        self.deposit_critical_region = ArenitoVision.DEPOSIT_CRITICAL_REGIONS[mode]

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
        b1 = Point(a1.x, self.r_dot.y)
        a2 = Point(self.bottom_center.x + t, self.bottom_center.y)
        b2 = Point(a2.x, self.r_dot.y)

        cv2.line(det_img, a1, b1, WHITE)
        cv2.line(det_img, a2, b2, WHITE)
        cv2.ellipse(det_img, self.r_dot, (t, t), 0.0, 180.0, 360.0, WHITE, thickness=1)

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
            (self.center_x_min, 0),
            (self.center_x_min, self.res_y),
            BLUE,
            thickness=1,
        )
        cv2.line(
            det_img,
            (self.center_x_max, 0),
            (self.center_x_max, self.res_y),
            BLUE,
            thickness=1,
        )

        for det in detections:
            cv2.circle(det_img, det.center, 10, WHITE, 10)
            cv2.drawContours(det_img, [det.contour], -1, GREEN, 1, cv2.LINE_AA) # pyright: ignore
            cv2.drawContours(det_img, [det.box], -1, RED, 1, cv2.LINE_AA) # pyright: ignore

        if dump:
            cv2.circle(det_img, dump.center, 10, ORANGE, 10)

    def dist_from_center(self, det: Point) -> float:
        """
        Calculates the distance from `self.bottom_center` to `det`.
        """

        x1, y1 = self.bottom_center
        x2, y2 = det

        return math.sqrt((x2 - x1)**2 + (y2 - y1)**2)

    def reachable(
        self,
        img_hsv: MatLike,
        det: Point,
    ) -> bool:
        """
        Determines if a detection is reachable. Returns true if possible, otherwise false.
        """

        mask_azul = ColorFilter.filter(img_hsv, ColorFilter.BLUE)
        mask_red = ColorFilter.filter(img_hsv, ColorFilter.RED)

        mask = cv2.bitwise_or(mask_azul, mask_red)

        line = np.zeros(shape=mask.shape, dtype=np.uint8)
        cv2.line(line, self.bottom_center, det, WHITE, thickness=self.vertical_line_thickness)
        cv2.rectangle(line, (0, self.bottom_line_y), (self.res_x, self.res_y), WHITE, thickness=-1)

        cross = cv2.bitwise_and(mask, line)
        white_px = np.count_nonzero(cross)

        # cv2.imshow("mask", mask)

        return white_px < self.min_px_water

    def min_rect_method(self, img: MatLike) -> list[Detection]:
        """
        Filters out cans by color and size utilizing cv2.minAreaRect().
        """

        # Without this cans that are on the border are invisible
        gray = cv2.copyMakeBorder(img, 1, 1, 1, 1, cv2.BORDER_CONSTANT, None, WHITE)

        # need better filter
        gray = cv2.cvtColor(gray, cv2.COLOR_RGB2GRAY)
        _, mask = cv2.threshold(gray, 50, 255, cv2.RETR_EXTERNAL)

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

            if w * h > self.min_can_area:
                det = Detection(rect, cnt)

                if self.reachable(img_hsv, det.center):
                    detections.append(det)

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
        return cv2.GaussianBlur(img, (51, 51), 0)

    def can_in_critical_region(self, detections: list[Detection]) -> bool:
        if not detections:
            return False

        return self.can_critical_region.point_inside(detections[0].center)

    def detect_dumping_zone(self, blurred_img: MatLike) -> None | Detection:
        """
        Filters out red color and returns a point indicating where it is.
        """

        # img = cv2.copyMakeBorder(blurred_img, 1, 1, 1, 1, cv2.BORDER_CONSTANT, None, WHITE)
        img_hsv = ColorFilter.filter(cv2.cvtColor(blurred_img, cv2.COLOR_BGR2HSV), ColorFilter.RED)

        contours, _ = cv2.findContours(img_hsv, cv2.RETR_TREE, cv2.CHAIN_APPROX_NONE)

        if not contours: return None

        rect = cv2.minAreaRect(contours[0])
        det = Detection(rect, contours[0])

        cv2.circle(blurred_img, det.center, 1, ORANGE, 1)
        cv2.drawContours(blurred_img, [det.box], -1, ORANGE, 1, cv2.LINE_AA) # pyright: ignore
        cv2.imshow('deposit', blurred_img)

        if cv2.waitKey(1) == 27:
            return None

        return det
