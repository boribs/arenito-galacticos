import Jetson.GPIO as GPIO

but_pin = 18

GPIO.setmode(GPIO.BOARD)
GPIO.setup(but_pin, GPIO.IN)

while True:
    print(GPIO.input(but_pin))
