#include "ArenitoUtils.h"

const int INSTRUCTION_EXECUTION_TIME = 100; // ms
const int MOTOR_PWM_ENABLE = 200;
const int BACKDOOR_PWM_UP = 130;
const int BACKDOOR_PWM_DOWN = 80;
const int BACKDOOR_TIMEOUT = 1000; // ms

// Don't use pin 13
IBT2 left = IBT2(12, 11);
IBT2 right = IBT2(10, 9);

enum InstructionMap {
    MoveForward = 'a',
    MoveLeft = 'i',
    MoveRight = 'd',
    MoveBack = 'r',
    MoveLongRight = 'D',
    RequestProxSensor = 's',
    DumpCans = 'c',
};

void setup() {
    left.setup();
    right.setup();

    Serial.begin(115200);
    Serial.setTimeout(0);

    Serial.println("hola");
}

void loop() {
    while (Serial.available() == 0) { ; }

    char instr = Serial.read();
    switch (instr) {

        case MoveForward:
            left.clockwise(MOTOR_PWM_ENABLE);
            right.clockwise(MOTOR_PWM_ENABLE);

            timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
                // measure distance with MPU6050
            });

            left.stop();
            right.stop();
            break;
    }

    Serial.println("aksjdfhas");
}
