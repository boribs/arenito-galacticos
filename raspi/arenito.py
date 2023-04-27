import serial
import cv2
import sys
import time
import argparse
import numpy as np
import desplazamiento as desp
from tflite_support.task import core
from tflite_support.task import processor
from tflite_support.task import vision


RES_X = 640
RES_Y = 380
MARGEN_X = 35 # Aquí para no tener que
              # modificarlo en más partes
CENTRO_INF = (RES_X // 2, RES_Y)
MIN_PX_WATER = 50

AZUL_LI = np.array([75, 160, 88], np.uint8)
AZUL_LS = np.array([112, 255, 255], np.uint8)

def reachable(img: np.ndarray, det: tuple[int]) -> bool:
    """
    Determines if a detection is reachable.
    Returns true if possible, otherwise false.
    """

    img_hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
    mask = cv2.inRange(img_hsv, AZUL_LI, AZUL_LS)

    line = np.zeros(shape=mask.shape, dtype=np.uint8)
    cv2.line(line, CENTRO_INF, det, (255, 255, 255), thickness=10)

    cross = cv2.bitwise_and(mask, line)
    white_px = np.count_nonzero(cross)

    return white_px >= MIN_PX_WATER

def find_cans(cap: cv2.VideoCapture, detector: vision.ObjectDetector) -> str:
    """
    Takes a picture when called and scans for cans.
    Returns a formatted detection list.
    """

    ok, img = cap.read()
    if not ok:
        sys.exit("Error leyendo la cámara.")

    rgb_img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
    input_tensor = vision.TensorImage.create_from_array(rgb_img)
    detections = detector.detect(input_tensor)
    imgname = f'{time.process_time()}.jpg'

    cv2.imwrite(imgname, img) # Guarda imagen limpia

    dets = []
    for det in detections.detections:
        bbox = det.bounding_box
        a = (bbox.origin_x, bbox.origin_y)
        b = (a[0] + bbox.width, a[1] + bbox.height)
        c = ((a[0] + b[0]) // 2, a[1]) # Solo se centra en X
                                       # para evitar problemas con la detección
                                       # del agua
        if reachable(img, c):
            dets.extend(c)
            cv2.rectangle(img, a, b, thickness=4, color=(0, 0, 255))
            # circulo rojo cuando no es alcanzable
        else:
            cv2.rectangle(img, a, b, thickness=4, color=(255, 0, 0))
            # circulo azul cuando es alcanzable

        cv2.circle(img, c, radius=5, thickness=4, color=(0, 0, 255))

    cv2.imwrite('det' + imgname, img) # Imagen con anotaciones de detecciones

    return '{' + ','.join(map(str, dets)) + ',}'

def main(
        port: str,
        baudrate: int,
        timeout: float,
        camera_id: int,
        model: str,
        num_threads: int,
        max_results: int,
        score_threshold: float,
    ):
    ser = serial.Serial(
        port, baudrate, timeout=timeout
    )  # Encontrar puerto automáticamente?
       # Recuerda dmesg | grep "tty"

    cap = cv2.VideoCapture(camera_id)
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, RES_X)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, RES_Y)

    base_options = core.BaseOptions(
        file_name=model, use_coral=False, num_threads=num_threads)
    detection_options = processor.DetectionOptions(
        max_results=max_results, score_threshold=score_threshold)
    options = vision.ObjectDetectorOptions(
        base_options=base_options, detection_options=detection_options)
    detector = vision.ObjectDetector.create_from_options(options)

    prev_rr = False
    while cap.isOpened():
        cap.read() # Es necesario estar haciendo esto constantemente?
        if cv2.waitKey(1) == 0: # Lee la documentación, por favor
            break

        msg = ser.readline().decode('utf-8').strip()
        if msg:
            print(msg)

        if msg.endswith('latas'):
            detections = find_cans(cap, detector)

            print(detections, end='')

            if detections == '{,}': # no encontró nada!
                detections = desp.select_destination(cap, prev_rr)
                prev_rr = detections == 'rr'
                print(' No encontró latas', end='')
            print()

            ser.write(bytes(detections, 'utf-8'))
        else:
            ser.write(b'{}')

    cap.release()

if __name__ == '__main__':
    parser = argparse.ArgumentParser() # Termina de llenar esto
    parser.add_argument(
        '--port',
        type=str,
        default='/dev/ttyUSB0',
    )
    parser.add_argument(
        '--baudrate',
        type=int,
        default=115200,
    )
    parser.add_argument(
        '--timeout',
        type=float,
        default=1.0,
    )
    parser.add_argument(
        '--camera_id',
        type=int,
        default=0,
    )
    parser.add_argument(
        '--model',
        type=str,
        default='./modelos/latas.tflite',
    )
    parser.add_argument(
        '--threads',
        type=int,
        default=4,
    )
    parser.add_argument(
        '--max_results',
        type=int,
        default=5,
    )
    parser.add_argument(
        '--score_threshold',
        type=float,
        default=0.6,
    )
    args = parser.parse_args()

    main(
        args.port,
        args.baudrate,
        args.timeout,
        args.camera_id,
        args.model,
        args.threads,
        args.max_results,
        args.score_threshold
    )
