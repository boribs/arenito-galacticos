# pyright: strict

import cv2
import math
import numpy as np
from typing import NamedTuple
from cv2.typing import MatLike, RotatedRect

class Point(NamedTuple):
    """
    A basic point implementation.
    """

    x: int
    y: int

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

class ColorFilter:
    """
    Stores color filter data.
    delete mongodb, flutter, sass
    """

    BLUE = (
        np.array([75, 160, 88]),   # lower
        np.array([175, 255, 255]), # upper
    )
    BLACK = (
        np.array([0, 0, 69]),      # lower
        np.array([175, 255, 255]), # upper
    )

class ArenitoVision:
    """
    This is where every vision-related operation will be handled.
    """

    def __init__(
        self,
        res_x: int,
        res_y: int,
        margin_x: int,
    ):
        self.res_x = res_x
        self.res_y = res_y

        # Bottom center of the image
        #
        # +------------------------+
        # |                        |
        # |                        |
        # |                        |
        # +------------X-----------+
        self.bottom_center = Point(res_x // 2, res_y)

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
        self.r_dot = Point(res_x // 2, res_y // 2 + self.water_dist_from_center)

        # Area limits where a detection is considered to be centered.
        # +------------------------+
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # +------------------------+
        self.margin_x = margin_x
        self.center_x_min = res_x // 2 - margin_x
        self.center_x_max = res_x // 2 + margin_x

        # When finding out if a point is reachable, counts how many blue pixels
        # there are between the robot and that point.
        # This is the minimum ammount of blue pixels necessary between the robot
        # and any given point for it to be considered `unreachable`.
        self.min_px_water = 50

        # Minimum size for a rect to be considered a can
        self.min_can_area = 500

        # Blob detector stuff
        params = cv2.SimpleBlobDetector.Params()
        params.filterByArea = True
        params.minArea = 500
        params.maxArea = 300000
        params.filterByCircularity = False
        params.filterByConvexity = False
        params.filterByInertia = True
        params.minInertiaRatio = 0.01
        params.maxInertiaRatio = 0.7

        self.blob_detector = cv2.SimpleBlobDetector.create(params)

    def add_markings(self, det_img: MatLike, detections: list[Detection]):
        """
        Adds visual markings to image to help visualize decisions.
        """

        WHITE = (255, 255, 255)

        t = 70
        a1 = Point(self.bottom_center.x - t, self.bottom_center.y)
        b1 = Point(a1.x, self.r_dot.y)
        a2 = Point(self.bottom_center.x + t, self.bottom_center.y)
        b2 = Point(a2.x, self.r_dot.y)

        cv2.line(det_img, a1, b1, WHITE)
        cv2.line(det_img, a2, b2, WHITE)
        cv2.ellipse(det_img, self.r_dot, (t, t), 0.0, 180.0, 360.0, WHITE, thickness=1)

        cv2.line(
            det_img,
            (0, self.res_y - 20),
            (self.res_x, self.res_y - 20),
            WHITE,
            thickness=1
        )
        cv2.line(
            det_img,
            (self.center_x_min, 0),
            (self.center_x_min, self.res_y),
            color=(255,0,0),
            thickness=1,
        )
        cv2.line(
            det_img,
            (self.center_x_max, 0),
            (self.center_x_max, self.res_y),
            color=(255,0,0),
            thickness=1,
        )

        for det in detections:
            cv2.circle(det_img, det.center, 10, (255, 255, 255), 10)
            cv2.drawContours(det_img, [det.contour], -1, (0, 255, 0), 1, cv2.LINE_AA) # pyright: ignore
            cv2.drawContours(det_img, [det.box], -1, (255, 0, 0), 1, cv2.LINE_AA) # pyright: ignore

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
        thickness: int = 140,
    ) -> bool:
        """
        Determines if a detection is reachable. Returns true if possible, otherwise false.
        """

        lower, upper = ColorFilter.BLUE
        mask_azul = cv2.inRange(img_hsv, lower, upper)

        line = np.zeros(shape=mask_azul.shape, dtype=np.uint8)
        cv2.line(line, self.bottom_center, det, (255, 255, 255), thickness=thickness)
        cv2.line(line, (0, self.res_y), (self.res_x, self.res_y), (255, 255, 255), thickness=40)

        cross = cv2.bitwise_and(mask_azul, line)
        white_px = np.count_nonzero(cross)

        # cv2.imshow('aaaa', mask_azul)

        return white_px < self.min_px_water

    def find_cans(self, img: MatLike) -> list[Detection]:
        """
        Filters out cans by color and size.
        This method will replace `ArenitoVision.find_blobs()`.
        """

        gray = cv2.cvtColor(img, cv2.COLOR_RGB2GRAY)
        _, thresh = cv2.threshold(gray, 50, 255, cv2.RETR_EXTERNAL)
        contours, _ = cv2.findContours(
            thresh,
            cv2.RETR_TREE,
            cv2.CHAIN_APPROX_NONE
        )

        # first one borders the whole frame
        contours = contours[1:]

        detections: list[Detection] = []
        for cnt in contours:
            rect = cv2.minAreaRect(cnt)
            w, h = rect[1]

            if w * h > self.min_can_area:
                det = Detection(rect, cnt)

                if self.reachable(img, det.center):
                    detections.append(det)

        return detections
