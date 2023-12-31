import cv2, cv2.typing
import numpy as np
import math
import argparse
from arenito_com import *

RES_X = 640
RES_Y = 380

# Centro inferior de la imagen
CENTRO_INF = None

# Posición del punto máximo para la tolerancia al agua
R_DOT = None

# Límites centrales para determinar si una lata está
# "en el centro"
CENTRO_X_MIN = None
CENTRO_X_MAX = None
MARGEN_X = None

WATER_TOLERANCE = 90

AZUL_LI = np.array([75, 160, 88], np.uint8)
AZUL_LS = np.array([179, 255, 255], np.uint8)
MIN_PX_WATER = 50

# Cuenta cuantas instrucciones lleva
# buscando latas
lr_count = 0
LR_COUNT_MAX = 20

def _dist(det: tuple[int]):
    x1, y1 = CENTRO_INF
    x2, y2 = det

    return math.sqrt((x2 - x1)**2 + (y2 - y1)**2)

def reachable(
        img_hsv: np.ndarray,
        det: tuple[int],
        thickness: int = 140,
    ) -> bool:
    """
    Determines if a detection is reachable.
    Returns true if possible, otherwise false.
    """

    mask_azul = cv2.inRange(img_hsv, AZUL_LI, AZUL_LS)

    line = np.zeros(shape=mask_azul.shape, dtype=np.uint8)
    cv2.line(line, CENTRO_INF, det, (255, 255, 255), thickness=thickness)
    cv2.line(line, (0, RES_Y), (RES_X, RES_Y), (255, 255, 255), thickness=40)

    cross = cv2.bitwise_and(mask_azul, line)
    white_px = np.count_nonzero(cross)

    # cv2.imshow('aaaa', mask_azul)

    return white_px < MIN_PX_WATER

def find_blobs(img: np.ndarray, detector: cv2.SimpleBlobDetector) -> np.ndarray:
    """
    Finds the positions of every can by applying a color filter to the image and
    calling SimpleBlobDetector's `detect()` method.

    Returns only reachable positions.
    """

    lower = np.array([0, 0, 69])
    upper = np.array([175, 255, 255])

    # Este borde es necesario porque sino no detecta las latas cerca
    # de las esquinas de la imagen
    img = cv2.copyMakeBorder(img, 1, 1, 1, 1, cv2.BORDER_CONSTANT, None, [255, 255, 255])

    hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
    mask = cv2.inRange(hsv, lower, upper)

    keypoints = detector.detect(mask)
    im_with_keypoints = cv2.drawKeypoints(img, keypoints, np.array([]), (0,0,255), cv2.DRAW_MATCHES_FLAGS_DRAW_RICH_KEYPOINTS)

    detections = []
    for k in keypoints:
        det = tuple(map(int, k.pt))
        if reachable(hsv, det):
            detections.append(det)
            cv2.circle(im_with_keypoints, det, 10, (255,0,0), 10)

    return im_with_keypoints, sorted(detections, key=_dist)

def send_move_instruction(com: ArenitoComms, det: tuple[int]):
    """
    Sends a move to left, right or forward instruction
    to the Arduino board, depending on the detection's position.
    """

    global lr_count

    x, _ = det

    if CENTRO_X_MAX <= x:
        com.send_instruction(Instruction.MOVE_LEFT)
    elif CENTRO_X_MIN >= x:
        com.send_instruction(Instruction.MOVE_RIGHT)
    else: # está centrado, avanza
        com.send_instruction(Instruction.MOVE_FORWARD)

    lr_count = 0

def send_roam_instruction(com: ArenitoComms, hsv_frame: np.ndarray):
    """
    Function strictly responsible for determining movement
    when no can detections are made.
    """

    global lr_count

    if reachable(hsv_frame, R_DOT):                   # si puede, avanza
        com.send_instruction(Instruction.MOVE_FORWARD)
    else:                                             # si no, gira
        com.send_instruction(Instruction.MOVE_RIGHT)

    lr_count += 1

    if lr_count == LR_COUNT_MAX:
        com.send_instruction(Instruction.MOVE_LONG_RIGHT)
        lr_count = 0

def get_image(com: ArenitoComms) -> cv2.typing.MatLike:
    return cv2.resize(com.get_image(), (RES_X, RES_Y), interpolation=cv2.INTER_LINEAR)

def main(com: ArenitoComms):
    global RES_X, RES_Y, CENTRO_INF, R_DOT, MARGEN_X, CENTRO_X_MIN, CENTRO_X_MAX

    params = cv2.SimpleBlobDetector_Params()
    params.filterByArea = True
    params.minArea = 500
    params.maxArea = 300000
    params.filterByCircularity = False
    params.filterByConvexity = False
    params.filterByInertia = True
    params.minInertiaRatio = 0.01
    params.maxInertiaRatio = 0.7

    detector = cv2.SimpleBlobDetector_create(params)

    # Cálculos relativos a la resolución de la imagen
    # solo se realizan una vez, al mero inicio
    CENTRO_INF = (RES_X // 2, RES_Y)

    R_DOT = (RES_X // 2, RES_Y // 2 + WATER_TOLERANCE)

    MARGEN_X = int(RES_X * 0.2)
    CENTRO_X_MIN = RES_X // 2 - MARGEN_X
    CENTRO_X_MAX = RES_X // 2 + MARGEN_X

    while True:
        frame = get_image(com)

        if cv2.waitKey(1) == 27:
            break

        det_img, detections = find_blobs(frame, detector)
        cv2.line(
            det_img,
            CENTRO_INF,
            R_DOT,
            (255, 255, 255),
            thickness=140
        )
        cv2.line(det_img, (0, RES_Y), (RES_X, RES_Y), (255, 255, 255), thickness=40)
        cv2.line(
            det_img,
            (CENTRO_X_MIN, 0),
            (CENTRO_X_MIN, RES_Y),
            color=(255,0,0),
            thickness=2,
        )
        cv2.line(
            det_img,
            (CENTRO_X_MAX, 0),
            (CENTRO_X_MAX, RES_Y),
            color=(255,0,0),
            thickness=2,
        )
        hsv_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
        cv2.imshow('asdf', det_img)

        if detections:
            det = detections[0]
            send_move_instruction(com, det)
        else:
            send_roam_instruction(com, hsv_frame)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('port', nargs='?', type=str, default=None)
    parser.add_argument('baudrate', nargs='?', type=int, default=115200)
    parser.add_argument('timeout', nargs='?', type=float, default=0.5)

    parser.add_argument('flink', nargs='?', type=str, default='../sim/shmem_arenito')
    parser.add_argument('--sim', '-s', action=argparse.BooleanOptionalAction, default=False)

    com = ArenitoComms()
    args = parser.parse_args()

    if args.sim:
        com.connect_simulation(args.flink)
    else:
        com.connect_serial(args.port, args.baudrate, args.timeout)
        com.init_video()

    try:
        main(com)
    except Exception as e:
        print(e)

    if com.sim_interface:
        com.sim_interface.close()
