#include "ArenitoUtils.h"

LimitSwitch a = LimitSwitch(23);
LimitSwitch b = LimitSwitch(24);

void setup() {
    a.setup();
    b.setup();

    pinMode(13, OUTPUT);

    Serial.begin(115200);
    Serial.setTimeout(0);

    Serial.println("hola");
}

void loop() {
    if (a.read() == LOW || b.read() == LOW) {
        digitalWrite(13, HIGH);
    } else {
        digitalWrite(13, LOW);
    }
}
