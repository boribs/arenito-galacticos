import Jetson.GPIO as GPIO

# Los botones deben conectarse bien
# https://docs.arduino.cc/built-in-examples/digital/Button/
# Recuerda poner una resistenica de 10K

but_pin = 18

GPIO.setmode(GPIO.BOARD)
GPIO.setup(but_pin, GPIO.IN)

while True:
    print(GPIO.input(but_pin))
