#include "ArenitoUtils.h"

DCMotor tapa = DCMotor(9, 8, 7);

void setup() {
  Serial.begin(9600);
  tapa.setup();
}

void loop() {
    if (digitalRead(4) != 0) {
        tapa.clockwise(130);
        delay(300);
    } else if (digitalRead(5) != 0) {
        tapa.counterClockwise(80);
        delay(150);
    } else {
        tapa.stop();
    }
}
