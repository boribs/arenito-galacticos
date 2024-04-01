import serial
import time

serial = serial.Serial(port='/dev/cu.usbserial-14210', baudrate=115200)

def rec():
    data = ''
    msg_complete = False

    while not msg_complete:
        c = serial.read().decode('utf-8')

        if c == '\r':
            msg_complete = True
        else:
            data += c

    return data

print(rec())

for _ in range(10):
    serial.write("h".encode('utf-8'))
    print(rec())
