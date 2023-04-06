import serial, time
from random import randint

ser = serial.Serial('/dev/ttyUSB0', 115200, timeout=0.1) # Encontrar esto automáticamente?

while True:
    msg = ser.seradline().decode('utf-8')
    if msg == 'latas':
        print('Enviando latas')
        time.sleep(randint(1, 3))
        ser.write(bytes('ok', 'utf-8'))
        print('Enviadas')
