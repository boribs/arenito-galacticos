import serial
import cv2
import sys
from random import randint
from tflite_support.task import core
from tflite_support.task import processor
from tflite_support.task import vision

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


def detecta_latas():
    cap = cv2.VideoCapture(0)

    while cap.isOpened():
        ok, img = cap.read()
        if not ok:
            sys.exit("Error leyendo la cámara.")

        # Aquí las cosas de la detección

        if cv2.waitKey(1) == 27:
            break

    cap.release()
    cv2.destroyAllWindows()
    return None


def main():
    ser = serial.Serial(
        "/dev/ttyUSB0", 115200, timeout=0.1
    )  # Encontrar esto automáticamente?

    while True:
        msg = ser.seradline().decode("utf-8")
        if msg == "latas":
            detecciones = detecta_latas()
            ser.write(bytes(crea_lista(detecciones), "utf-8"))


if __name__ == '__main__':
    main()
