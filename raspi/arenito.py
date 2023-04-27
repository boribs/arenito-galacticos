import serial
import cv2
import sys
import time
import argparse
import numpy as np


def process_img(img: np.ndarray) -> np.ndarray:
    lower = np.array([0, 0, 69])
    upper = np.array([175, 255, 255])

    hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
    mask = cv2.inRange(hsv, lower, upper)

    return mask

def find_blobs(mask: np.ndarray) -> np.ndarray:
    params = cv2.SimpleBlobDetector_Params()
    params.filterByArea = True
    params.minArea = 500
    params.maxArea = 30000
    params.filterByCircularity = False
    params.filterByConvexity = False
    params.filterByInertia = False

    detector = cv2.SimpleBlobDetector_create(params)
    keypoints = detector.detect(mask)
    im_with_keypoints = cv2.drawKeypoints(mask, keypoints, np.array([]), (0,0,255), cv2.DRAW_MATCHES_FLAGS_DRAW_RICH_KEYPOINTS)
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
