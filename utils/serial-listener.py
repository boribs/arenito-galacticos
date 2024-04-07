import serial

port = '/dev/cu.usbserial-14240'
ser = serial.Serial(port, 115200)

while True:
    msg = ser.readline().decode('utf-8').strip()
    if msg:
        print(msg)
