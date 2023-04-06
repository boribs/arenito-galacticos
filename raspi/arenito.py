import serial, time

ser = serial.Serial('/dev/ttyUSB0') # Encontrar esto automáticamente?
ser.write(bytes('8', 'utf-8'))
