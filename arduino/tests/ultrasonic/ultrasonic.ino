#include "ArenitoUtils.h"

Ultrasonic u1 = Ultrasonic(2, 3);
Ultrasonic u2 = Ultrasonic(4, 5);
Ultrasonic u3 = Ultrasonic(6, 7);
Ultrasonic u4 = Ultrasonic(8, 9);

void setup() {
    pinMode(13, OUTPUT);
    u1.setup();
    u2.setup();
    u3.setup();
    u4.setup();
}

void loop() {
    if (
        u1.read() < 15 ||
        u2.read() < 15 ||
        u3.read() < 15 ||
        u4.read() < 15
    ) {
        digitalWrite(13, HIGH);
    } else {
        digitalWrite(13, LOW);
    }

    // delay(100);
}
