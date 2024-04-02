#include "ArenitoUtils.h"

DCMotor tapa = DCMotor(7, 6);

void setup() {
  Serial.begin(9600);
  tapa.setup();
}

void loop() {
    if (digitalRead(4) != 0) {
        tapa.clockwise();
    } else if (digitalRead(5) != 0) {
        tapa.counterClockwise();
    } else {
        tapa.stop();
    }
}
