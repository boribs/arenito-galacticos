import serial
import cv2
import sys
import time
from random import randint
from tflite_support.task import core
from tflite_support.task import processor
from tflite_support.task import vision

i = 0

def crea_lista(detecciones: processor.DetectionResult) -> str:
    """
    Crea una cadena con las detecciones:
    "{x1,y1,x2,y2,...,xn,yn}"

    Donde cada par x, y corresponde al punto medio del rectángulo
    que encierra cada lata detectada.
    """

    salida = "{"
    for det in detecciones.detections:
        bbox = det.bounding_box
        a = (bbox.origin_x, bbox.origin_y)
        b = (a[0] + bbox.width, a[1] + bbox.height)
        salida += f"{a[0] + b[0] // 2},{a[1] + b[1] // 2},"

    return salida + "}"

def detecta_latas(model: str, cap: cv2.VideoCapture) -> str:
    """
    Toma una foto y detecta latas.

    Regresa una cadena formateada con las detecciones de las latas.
    """

    ok, img = cap.read()
    if not ok:
        sys.exit("Error leyendo la cámara.")

    base_options = core.BaseOptions(
        file_name=model, use_coral=False, num_threads=4)
    detection_options = processor.DetectionOptions(
        max_results=3, score_threshold=0.3)
    options = vision.ObjectDetectorOptions(
        base_options=base_options, detection_options=detection_options)
    detector = vision.ObjectDetector.create_from_options(options)

    rgb_img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
    input_tensor = vision.TensorImage.create_from_array(rgb_img)
    detecciones = detector.detect(input_tensor)

    cv2.imwrite(f'det{time.process_time()}.jpg', img)

    return crea_lista(detecciones)

def main():
    # ser = serial.Serial(
    #     "/dev/ttyUSB0", 115200, timeout=0.1
    # )  # Encontrar esto automáticamente?

    cap = cv2.VideoCapture(0)

    # while True:
    #     msg = ser.seradline().decode("utf-8")
    #     if msg == "latas":
    #         detecciones = detecta_latas('latas.tflite', cap)
    #         ser.write(bytes(crea_lista(detecciones), "utf-8"))

    detecta_latas('latas.tflite', cap)


if __name__ == '__main__':
    main()
