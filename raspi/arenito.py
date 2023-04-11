import serial
import cv2
import sys
import time
from random import randint
from tflite_support.task import core
from tflite_support.task import processor
from tflite_support.task import vision

def detecta_latas(cap: cv2.VideoCapture, detector: vision.ObjectDetector) -> str:
    """
    Toma una foto y detecta latas.

    Regresa una cadena formateada con las detecciones de las latas.
    """

    ok, img = cap.read()
    if not ok:
        sys.exit("Error leyendo la cámara.")

    rgb_img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
    input_tensor = vision.TensorImage.create_from_array(rgb_img)
    detecciones = detector.detect(input_tensor)

    dets = []
    for det in detecciones.detections:
        bbox = det.bounding_box
        a = (bbox.origin_x, bbox.origin_y)
        b = (a[0] + bbox.width, a[1] + bbox.height)
        c = (a[0] + b[0] // 2, a[1] + b[1] // 2)
        dets.extend(c)
        cv2.circle(img, c, radius=5, color=(255, 0, 0))

    cv2.imwrite(f'det{time.process_time()}.jpg', img)

    return '{' + ','.join(map(str, dets)) + ',}'

def main():
    # ser = serial.Serial(
    #     "/dev/ttyUSB0", 115200, timeout=0.1
    # )  # Encontrar esto automáticamente?

    cap = cv2.VideoCapture(0)
    model = 'latas.tflite'

    base_options = core.BaseOptions(
        file_name=model, use_coral=False, num_threads=4)
    detection_options = processor.DetectionOptions(
        max_results=3, score_threshold=0.3)
    options = vision.ObjectDetectorOptions(
        base_options=base_options, detection_options=detection_options)
    detector = vision.ObjectDetector.create_from_options(options)

    # while True:
    #     msg = ser.seradline().decode("utf-8")
    #     if msg == "latas":
    #         detecciones = detecta_latas('latas.tflite', cap)
    #         ser.write(bytes(crea_lista(detecciones), "utf-8"))

    detecta_latas(cap, detector)


if __name__ == '__main__':
    main()
