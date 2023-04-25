import serial
import argparse
import cv2
import uuid
from typing import Union

# El movimiento del arduino depende de las detecciones.
# Actualmente, si tiene una lata "en frente", avanza hacia adelante.
# Si la tiene a un lado, gira hacia ella.
#
# Este script simula haber encontrado detecciones, permitiendo controlar
# manualmente el movimiento del arenito.
#
# Se considera "en frente" aquellas detecciones que, en el plano horizontal,
# se encuentren en el rango de RES_X / 2 ± MARGEN_X.
# RES_X y MARGEN_X están definidos en arenito.py, con el objetivo de
# reducir los archivos a modificar para realizar cambios

from arenito import RES_X, RES_Y, MARGEN_X

CENTRO = RES_X // 2
LIMITES = (CENTRO - MARGEN_X, CENTRO + MARGEN_X)

def lerp(a: int, b: int, t: int):
    return int(a + ((b - a) * (t / 100)))

def adelante(ser: serial.Serial, perc: Union[int, None]):
    if perc != None:
        ser.write(bytes(
            f'{{{CENTRO},{lerp(0, RES_Y, perc)},}}'
            'utf-8'
        ))
    else:
        ser.write(bytes('{' + str(CENTRO) + ',200,}', 'utf-8'))

def izquierda(ser: serial.Serial, perc: Union[int, None]):
    if perc != None:
        ser.write(bytes(
            f'{{{LIMITES[0] - lerp(0, LIMITES[0], perc)},200,}}'
            'utf-8'
        ))
    else:
        ser.write(bytes('{0,200,}', 'utf-8'))

def derecha(ser: serial.Serial, perc: Union[int, None]):
    if perc != None:
        ser.write(bytes(
            f'{{{lerp(LIMITES[1], RES_X, perc)},200,}}'
            'utf-8'
        ))
    else:
        ser.write(bytes('{' + str(RES_X) + ',200,}', 'utf-8'))

def foto(camera_id: int):
    try:
        cam = cv2.VideoCapture(camera_id)
        ok, frame = cam.read()
        if not ok:
            print('Error tomando foto')

        imgname = f'{uuid.uuid1()}.jpg'
        cv2.imwrite(str(imgname, frame))
        print(f'Guardado {imgname}')

    except Exception as e:
        print(f'No se puede tomar foto:', e)


def main(port: str, baudrate: int, timeout: float, camera_id: int):
    print('Iniciando control manual...')
    ser = serial.Serial(port, baudrate, timeout=timeout)
    cam = True

    try:
        cv2.VideoCapture(camera_id)
    except Exception:
        print('No es posible comunicarse con la cámara.')
        cam = False

    while True:
        try:
            perc = None
            dire = None
            cmd = input('> ').strip().split()

            if len(cmd) == 2:
                dire, perc = cmd
                perc = int(perc)

                if perc > 100 or perc < 1:
                    raise Exception('Porcentaje debe estar entre 1 y 100.')

            elif len(cmd) == 1:
                dire = cmd[0]
            else:
                raise Exception('Sintaxis inválida.')

            dire = dire.lower()
            if dire in ('exit', 'q', 'quit', 'salir', 's', 'bye'):
                raise KeyboardInterrupt

            # TODO: Comando para tomar foto?
            if dire == 'a':
                adelante(ser, perc)
            elif dire == 'i':
                izquierda(ser, perc)
            elif dire == 'd':
                derecha(ser, perc)
            elif dire == 'f':
                if cam:
                    foto(camera_id)
                else:
                    print('No hay comunicación con la cámara.')
            else:
                raise Exception('Comando inválido.')

        except KeyboardInterrupt:
            print('\nSaliendo')
            break

        except Exception as e:
            print(f'{e}\nRecuerda: [direccion] [porcentaje?]\n')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
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
        default=0.1,
    )
    parser.add_argument(
        '--camera_id',
        type=int,
        default=0,
    )
    args = parser.parse_args()

    main(args.port, args.baudrate, args.timeout, args.camera_id)
