import cv2
import math
import numpy as np
from typing import TypeAlias
from cv2.typing import MatLike
from collections import namedtuple

Point: TypeAlias = namedtuple('Point', ('x', 'y'))

class ArenitoVision:
    """
    This is where every vision-related operation will be handled.
    """

    def __init__(
        self,
        res_x: int,
        res_y: int,
        margen_x: int,
    ):
        # TODO: Rename all these variables

        self.res_x = res_x
        self.res_y = res_y

        # Bottom center of the image
        #
        # +------------------------+
        # |                        |
        # |                        |
        # |                        |
        # +------------X-----------+
        self.centro_inf = (res_x // 2, res_y)

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
        self.r_dot = (res_x // 2, res_y // 2 + self.water_dist_from_center)

        # Area limits where a detection is considered to be centered.
        # +------------------------+
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # |          |   |         |
        # +------------------------+
        self.margen_x = margen_x
        self.centro_x_min = res_x // 2 - margen_x
        self.centro_x_max = res_x // 2 + margen_x

        # Blue color filter parameters
        self.azul_li = np.array([75, 160, 88], np.uint8)
        self.azul_ls = np.array([179, 255, 255], np.uint8)
        # When finding out if a point is reachable, counts how many blue pixels
        # there are between the robot and that point.
        # This is the minimum ammount of blue pixels necessary between the robot
        # and any given point for it to be considered `unreachable`.
        self.min_px_water = 50

        # Blob detector stuff
        params = cv2.SimpleBlobDetector_Params()
        params.filterByArea = True
        params.minArea = 500
        params.maxArea = 300000
        params.filterByCircularity = False
        params.filterByConvexity = False
        params.filterByInertia = True
        params.minInertiaRatio = 0.01
        params.maxInertiaRatio = 0.7

        self.blob_detector: cv2.SimpleBlobDetector = cv2.SimpleBlobDetector_create(params)

    def add_markings(self, det_img: MatLike):
        """
        Adds visual markings to image to help visualize decisions.
        """

        cv2.line(
            det_img,
            self.centro_inf,
            self.r_dot,
            (255, 255, 255),
            thickness=140
        )
        cv2.line(
            det_img,
            (0, self.res_y),
            (self.res_x, self.res_y),
            (255, 255, 255),
            thickness=40
        )
        cv2.line(
            det_img,
            (self.centro_x_min, 0),
            (self.centro_x_min, self.res_y),
            color=(255,0,0),
            thickness=2,
        )
        cv2.line(
            det_img,
            (self.centro_x_max, 0),
            (self.centro_x_max, self.res_y),
            color=(255,0,0),
            thickness=2,
        )

    def dist_from_center(self, det: Point):
        """
        Calculates the distance from `self.centro_inf` to `det`.
        """

        x1, y1 = self.centro_inf
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

        mask_azul = cv2.inRange(img_hsv, self.azul_li, self.azul_ls)

        line = np.zeros(shape=mask_azul.shape, dtype=np.uint8)
        cv2.line(line, self.centro_inf, det, (255, 255, 255), thickness=thickness)
        cv2.line(line, (0, self.res_y), (self.res_x, self.res_y), (255, 255, 255), thickness=40)

        cross = cv2.bitwise_and(mask_azul, line)
        white_px = np.count_nonzero(cross)

        # cv2.imshow('aaaa', mask_azul)

        return white_px < self.min_px_water

    def find_blobs(self, img: MatLike) -> tuple[MatLike, list[Point]]:
        """
        Finds the positions of every can by applying a color filter to the image and
        calling SimpleBlobDetector's `detect()` method.

        Returns only reachable positions.
        TODO: Parameter to enable circle drawing on reachable elements.
        """

        lower = np.array([0, 0, 69])
        upper = np.array([175, 255, 255])

        # Este borde es necesario porque sino no detecta las latas cerca
        # de las esquinas de la imagen
        img = cv2.copyMakeBorder(img, 1, 1, 1, 1, cv2.BORDER_CONSTANT, None, [255, 255, 255])

        hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
        mask = cv2.inRange(hsv, lower, upper)

        keypoints = self.blob_detector.detect(mask)
        im_with_keypoints = cv2.drawKeypoints(
            img,
            keypoints,
            np.array([]),
            (0, 0, 255),
            cv2.DRAW_MATCHES_FLAGS_DRAW_RICH_KEYPOINTS
        )

        detections = []
        for k in keypoints:
            det = tuple(map(int, k.pt))
            if self.reachable(hsv, det):
                detections.append(det)
                cv2.circle(im_with_keypoints, det, 10, (255, 0, 0), 10)

        return im_with_keypoints, sorted(detections, key=self.dist_from_center)
