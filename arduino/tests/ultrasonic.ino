#include "ArenitoUtils.h"

u1 = Ultrasonic(2, 3);

void setup() {
    pinMode(13, OUTPUT);
    u1.setup();
}

void loop() {
    if (u1.read() < 15) {
        digitalWrite(13, HIGH);
    } else {
        digitalWrite(13, LOW);
    }

    // delay(100);
}
