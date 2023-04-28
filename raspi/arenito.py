import serial
import cv2
import sys
import time
import argparse
import numpy as np
import math
from enum import Enum, auto

MARGEN_X = 35 # Aquí para no tener que
              # modificarlo en más partes
RES_X = None
RES_Y = None
CENTRO_INF = None
R_DOT = None

AZUL_LI = np.array([75, 160, 88], np.uint8)
AZUL_LS = np.array([179, 255, 255], np.uint8)
MIN_PX_WATER = 50

class Instruction(Enum):
    FORWARD = auto()
    LEFT = auto()
    RIGHT = auto()
    BACK = auto()

INSTRUCTION_MAP = {
    Instruction.FORWARD: 'a',
    Instruction.LEFT: 'i',
    Instruction.RIGHT: 'd',
    Instruction.BACK: 'r',
}

def _dist(det: tuple[int]):
    x1, y1 = CENTRO_INF
    x2, y2 = det

    return math.sqrt((x2 - x1)**2 + (y2 - y1)**2)

def reachable(img_hsv: np.ndarray, det: tuple[int], thickness: int = 10) -> bool:
    """
    Determines if a detection is reachable.
    Returns true if possible, otherwise false.
    """

    mask = cv2.inRange(img_hsv, AZUL_LI, AZUL_LS)

    line = np.zeros(shape=mask.shape, dtype=np.uint8)
    cv2.line(line, CENTRO_INF, det, (255, 255, 255), thickness=thickness)

    cross = cv2.bitwise_and(mask, line)
    white_px = np.count_nonzero(cross)

    # cv2.imwrite('w.jpg', mask)
    # cv2.imwrite('x.jpg', line)
    # cv2.imwrite('y.jpg', cross)

    return white_px < MIN_PX_WATER

def find_blobs(img: np.ndarray, detector: cv2.SimpleBlobDetector) -> np.ndarray:
    lower = np.array([0, 0, 69])
    upper = np.array([175, 255, 255])

    hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
    mask = cv2.inRange(hsv, lower, upper)

    # cv2.imwrite('mask.jpg', mask)

    keypoints = detector.detect(mask)
    im_with_keypoints = cv2.drawKeypoints(img, keypoints, np.array([]), (0,0,255), cv2.DRAW_MATCHES_FLAGS_DRAW_RICH_KEYPOINTS)

    detections = []
    for k in keypoints:
        det = tuple(map(int, k.pt))
        if reachable(hsv, det):
            detections.append(det)
            cv2.circle(im_with_keypoints, det, 10, (255,0,0), 10)

    return im_with_keypoints, sorted(detections, key=_dist)

def _send_serial_instr(ser: serial.Serial, instr: Instruction):
    """
    Function that converts the instruction type to a
    stream of characters, readable by the Arduino board.
    """
    p = ser.read()
    if p:
        print(f'Enviando {INSTRUCTION_MAP[instr]}::{p}')
        ser.write(bytes(
            INSTRUCTION_MAP[instr],
            'utf-8'
        ))

def send_move_instruction(ser: serial.Serial, det: tuple[int]):
    pass

def send_roam_instruction(ser: serial.Serial, hsv_frame: np.ndarray):
    """
    Function strictly responsible for determining movement
    when no can detections are made.
    """

    if reachable(hsv_frame, R_DOT):                   # si puede, avanza
        _send_serial_instr(ser, Instruction.FORWARD)
    else:                                             # si no, gira
        _send_serial_instr(ser, Instruction.RIGHT)

def main():
    global RES_X, RES_Y, CENTRO_INF, R_DOT

    cap = cv2.VideoCapture(0)
    params = cv2.SimpleBlobDetector_Params()
    params.filterByArea = True
    params.minArea = 500
    params.maxArea = 300000
    params.filterByCircularity = False
    params.filterByConvexity = False
    params.filterByInertia = False
    detector = cv2.SimpleBlobDetector_create(params)

    ser = serial.Serial(
        port='/dev/cu.usbmodem142101',
        baudrate=115200,
        timeout=0.1,
    )

    _, frame = cap.read()
    RES_Y, RES_X, _ = frame.shape
    CENTRO_INF = (RES_X // 2, RES_Y)
    R_DOT = (RES_X // 2, RES_Y // 2)

    while True:
        ok, frame = cap.read()

        if not ok:
            print('error')
            break

        if cv2.waitKey(1) == 27:
            break

        det_img, detections = find_blobs(frame, detector)
        # cv2.circle(frame, , radius=100, color=(0,255,0), thickness=-1)
        hsv_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
        cv2.imshow('asdf', det_img)

        if detections:
            det = detections[0]
            send_move_instruction(ser, det)
        else:
            send_roam_instruction(ser, hsv_frame)

        # time.sleep(0.1)
    # frame = cv2.imread(sys.argv[1])
    # frame, _ = find_blobs(frame, detector)
    # cv2.imwrite('asdf.jpg', frame)

main()
