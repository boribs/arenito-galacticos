#include "ArenitoUtils.h"

IrProx a = IrProx(2);

void setup() {
    a.setup();
    pinMode(13, OUTPUT);
}

void loop() {
    digitalWrite(13, a.inRange() ? HIGH : LOW);
}
