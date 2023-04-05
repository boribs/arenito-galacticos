import serial

CONF = 0xFF

ser = serial.Serial('/dev/ttyUSB0') # Encontrar esto automáticamente?
ser.write(CONF)
ser.write(0)
