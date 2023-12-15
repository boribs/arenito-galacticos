import serial
import cv2, cv2.typing
import subprocess
import numpy as np
import math
import argparse
from enum import Enum, auto
from PIL import Image

INSTRUCTION_PIPE_PATH = '../pipes/instrpipe'
IMAGE_PIPE_PATH = '../pipes/imgpipe'

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

class Instruction(Enum):
    FORWARD = auto()
    LEFT = auto()
    RIGHT = auto()
    BACK = auto()
    LONG_RIGHT = auto()
    SCREENSHOT = auto()

INSTRUCTION_MAP = {
    Instruction.FORWARD: 'a',
    Instruction.LEFT: 'i',
    Instruction.RIGHT: 'd',
    Instruction.BACK: 'r',
    Instruction.LONG_RIGHT: 'l',
    Instruction.SCREENSHOT: 'ss',
}

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

def _send_instr(ser: serial.Serial | None, instr: Instruction):
    """
    Function that converts the instruction type to a
    stream of characters, readable by the Arduino board.
    """

    if isinstance(ser, serial.Serial):
        p = ser.read()
        if p:
            print(f'Enviando {INSTRUCTION_MAP[instr]}::{p}')
            ser.write(bytes(
                INSTRUCTION_MAP[instr],
                'utf-8'
            ))
    else:
        with open(INSTRUCTION_PIPE_PATH, 'w') as pout:
            print(f'Enviando {INSTRUCTION_MAP[instr]}')
            pout.write(f'mv:{INSTRUCTION_MAP[instr]}')

def send_move_instruction(ser: serial.Serial | None, det: tuple[int]):
    """
    Sends a move to left, right or forward instruction
    to the Arduino board, depending on the detection's position.
    """

    global lr_count

    x, _ = det

    if CENTRO_X_MAX <= x:
        _send_instr(ser, Instruction.LEFT)
    elif CENTRO_X_MIN >= x:
        _send_instr(ser, Instruction.RIGHT)
    else:
        _send_instr(ser, Instruction.FORWARD) # está centrado, avanza

    lr_count = 0

def send_roam_instruction(ser: serial.Serial | None, hsv_frame: np.ndarray):
    """
    Function strictly responsible for determining movement
    when no can detections are made.
    """
    global lr_count

    if reachable(hsv_frame, R_DOT):                   # si puede, avanza
        _send_instr(ser, Instruction.FORWARD)
    else:                                             # si no, gira
        _send_instr(ser, Instruction.RIGHT)

    lr_count += 1

    if lr_count == LR_COUNT_MAX:
        _send_instr(ser, Instruction.LONG_RIGHT)
        lr_count = 0

def get_image(mode: str, cap: cv2.VideoCapture | None) -> tuple[bool, cv2.typing.MatLike]:
    if mode != 'sim':
        ok, frame = cap.read()
        frame = cv2.resize(frame, (RES_X, RES_Y), interpolation=cv2.INTER_LINEAR)
        return (ok, frame)
    else:
        with open(INSTRUCTION_PIPE_PATH, 'w') as pout:
            pout.write(INSTRUCTION_MAP[Instruction.SCREENSHOT])

        with open(IMAGE_PIPE_PATH, 'rb') as pin:
            im = Image.frombytes('RGB', (1024, 1024), pin.read())
            return True, np.array(im)

def find_port() -> str:
    """
    Finds out where the Arduino borad is connected. Requires `arduino-cli`.
    """

    out = subprocess.run(["arduino-cli", "board", "list"], capture_output=True, text=True)
    ports = []
    for line in out.stdout.split('\n')[1:]:
        if line:
            line = list(map(lambda n: n.strip(), line.split()))
            if 'Unknown' not in line:
                ports.append(line)

    return ports[0][0]

def main(mode: str):
    global RES_X, RES_Y, CENTRO_INF, R_DOT, MARGEN_X, CENTRO_X_MIN, CENTRO_X_MAX

    cap = cv2.VideoCapture(0)
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

    if mode != 'sim':
        ser = serial.Serial(
            port=mode,
            baudrate=115200,
            timeout=0.05,
        )
    else:
        ser = None

    # Cálculos relativos a la resolución de la imagen
    # solo se realizan una vez, al mero inicio
    CENTRO_INF = (RES_X // 2, RES_Y)

    R_DOT = (RES_X // 2, RES_Y // 2 + WATER_TOLERANCE)

    MARGEN_X = int(RES_X * 0.2)
    CENTRO_X_MIN = RES_X // 2 - MARGEN_X
    CENTRO_X_MAX = RES_X // 2 + MARGEN_X

    while True:
        ok, frame = get_image(mode, cap)

        if not ok:
            print('error')
            break

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
            send_move_instruction(ser, det)
        else:
            send_roam_instruction(ser, hsv_frame)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('port', nargs='?')
    parser.add_argument('--sim', '-s', action=argparse.BooleanOptionalAction, default=False)

    args = parser.parse_args()
    if args.sim:
        port = 'sim'
    else:
        port = args.port or find_port()

    main(port)
