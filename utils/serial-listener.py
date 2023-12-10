import serial

ser = serial.Serial('/dev/ttyACM0', 115200, timeout=1)

while True:
    msg = ser.readline().decode('utf-8').strip()
    if msg:
        print(msg)
