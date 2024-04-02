#include "ArenitoUtils.h"

DCMotor tapa = DCMotor(9, 8, 7);
const int ls_sup = 11;
// const int ls_inf = 10;
const int up = 4;
// const int dw = 5;

void setup() {
    Serial.begin(9600);
    tapa.setup();
    pinMode(up, INPUT);
    pinMode(dw, INPUT);
    pinMode(ls_sup, INPUT);
    // pinMode(ls_inf, INPUT);

    pinMode(13, OUTPUT);
}

void loop() {
    if (digitalRead(up) == LOW) {
        tapa.clockwise(130);
        digitalWrite(13, HIGH);

        timeout_repeat(1500, []() {
            return digitalRead(ls_sup) == LOW;
        });

        digitalWrite(13, LOW);
        tapa.stop();
    }
}
