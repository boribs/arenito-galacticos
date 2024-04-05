#include "ArenitoUtils.h"

IBT2 left = IBT2(9, 8);

void setup() {
    left.setup();

    Serial.begin(115200);
    Serial.setTimeout(0);

    Serial.println("hola");
}

void loop() {
    while (Serial.available() == 0) { ; }

    char instr = Serial.read();
    switch (instr) {
        case 'a':
            left.clockwise(200);
            delay(100);
            left.stop();
            break;
    }
}
