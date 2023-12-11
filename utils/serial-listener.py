from compile import find_arduino
import serial

port, _ = find_arduino()
ser = serial.Serial(port, 115200, timeout=1)

while True:
    msg = ser.readline().decode('utf-8').strip()
    if msg:
        print(msg)
