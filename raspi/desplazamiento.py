# Este script es exclusivo para la prueba de movimiento en la arena,
# evitando entrar al auga, durante 40 segundos.

import serial
import cv2
import argparse
import sys
import numpy as np
from random import randint

# from arenito import RES_X, RES_Y, MARGEN_X, reachable

RES_X = 640
RES_Y = 380
CENTRO_INF = (RES_X // 2, RES_Y)
MIN_PX_WATER = 50
AZUL_LI = np.array([75, 160, 88], np.uint8)
AZUL_LS = np.array([112, 255, 255], np.uint8)

RECT_MARGIN_X = 60
RECT_MARGIN_Y = 40
RECT = (RECT_MARGIN_X, RECT_MARGIN_Y, RES_X - RECT_MARGIN_X, RES_Y - RECT_MARGIN_Y)

def valid(img: np.ndarray, det: tuple[int]) -> bool:
    """
    Determines if a given point is valid.
    Returns true if possible, otherwise false.
    """

    img_hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
    mask = cv2.inRange(img_hsv, AZUL_LI, AZUL_LS)
    cv2.imwrite('mask.jpg', mask)

    circle = np.zeros(shape=mask.shape, dtype=np.uint8)
    cv2.circle(circle, det, radius=50, color=(255, 255, 255), thickness=-1)
    cv2.imwrite('circle.jpg', circle)

    cross = cv2.bitwise_and(mask, circle)
    cv2.imwrite('cross.jpg', cross)

    white_px = np.count_nonzero(cross)

    return white_px == 0

def select_destination(cap: cv2.VideoCapture):
    ok, img = cap.read()

    cv2.rectangle(img, (RECT[0], RECT[1]), (RECT[2], RECT[3]), color=(255, 0, 0))
    cv2.imwrite('dest.jpg', img)

    if not ok:
        sys.exit('Error con la cámara.')

    while True:
        x = randint(RECT[0], RECT[2])
        y = randint(RECT[1], RECT[3])

        if valid(img, (x, y)):
            break

    return f'{{{x}, {y},}}'

def main(
        port: str,
        baudrate: int,
        timeout: float,
        camera_id: int,
    ):

    # ser = serial.Serial(
    #     port, baudrate, timeout=timeout
    # )
    ser = None

    cap = cv2.VideoCapture(camera_id)
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, RES_X)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, RES_Y)

    while cap.isOpened():
        cap.read()
        if cv2.waitKey(1) == 0:
            break

        # msg = ser.readline().decode('utf-8').strip()
        # if msg == 'latas':
        dest = select_destination(cap)
        print(dest)
        break
            # ser.write(bytes(dest, 'utf-8'))
        # else:

    cap.release()

if __name__ == '__main__':
    main(0, 0, 0, 0)
