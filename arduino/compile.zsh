# Recuerda, esto es lo que necesitas para compilar y subir...
# https://arduino.github.io/arduino-cli/0.32/getting-started/

arduino-cli compile -p /dev/cu.usbmodem141201 --fqbn arduino:avr:mega /Users/boristoteles/Documents/tmr23/arenito/arduino/arenito/arenito.ino
arduino-cli upload -p /dev/cu.usbmodem141201 --fqbn arduino:avr:mega /Users/boristoteles/Documents/tmr23/arenito/arduino/arenito/arenito.ino
