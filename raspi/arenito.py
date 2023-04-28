import serial
import cv2
import sys
import time
import argparse
import numpy as np

MARGEN_X = 35 # Aquí para no tener que
              # modificarlo en más partes
MIN_PX_WATER = 50

AZUL_LI = np.array([75, 160, 88], np.uint8)
AZUL_LS = np.array([179, 255, 255], np.uint8)

def reachable(img: np.ndarray, det: tuple[int]) -> bool:
    """
    Determines if a detection is reachable.
    Returns true if possible, otherwise false.
    """

    RES_Y, RES_X, _ = img.shape
    CENTRO_INF = (RES_X // 2, RES_Y)

    img_hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
    mask = cv2.inRange(img_hsv, AZUL_LI, AZUL_LS)

    line = np.zeros(shape=mask.shape, dtype=np.uint8)
    cv2.line(line, CENTRO_INF, det, (255, 255, 255), thickness=10)

    cross = cv2.bitwise_and(mask, line)
    white_px = np.count_nonzero(cross)

    cv2.imwrite('w.jpg', mask)
    cv2.imwrite('x.jpg', line)
    cv2.imwrite('y.jpg', cross)

    return white_px < MIN_PX_WATER

def find_blobs(img: np.ndarray) -> np.ndarray:
    params = cv2.SimpleBlobDetector_Params()
    params.filterByArea = True
    params.minArea = 500
    params.maxArea = 30000
    params.filterByCircularity = False
    params.filterByConvexity = False
    params.filterByInertia = False

    lower = np.array([0, 0, 69])
    upper = np.array([175, 255, 255])

    hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
    mask = cv2.inRange(hsv, lower, upper)

    detector = cv2.SimpleBlobDetector_create(params)
    keypoints = detector.detect(mask)
    im_with_keypoints = cv2.drawKeypoints(img, keypoints, np.array([]), (0,0,255), cv2.DRAW_MATCHES_FLAGS_DRAW_RICH_KEYPOINTS)

    for k in keypoints:
        det = tuple(map(int, k.pt))
        if reachable(img, det):
            cv2.circle(im_with_keypoints, det, 10, (255,0,0), 10)

    return im_with_keypoints

def main():
    cap = cv2.VideoCapture(0)

    while True:
        ok, frame = cap.read()

        if not ok:
            print('error')
            break

        if cv2.waitKey(1) == 27:
            break

        mask = process_img(frame)
        frame = find_blobs(mask)
        cv2.imshow('asdf', frame)

main()
