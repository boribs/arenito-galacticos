#include "ArenitoUtils.h"

const int INSTRUCTION_EXECUTION_TIME = 1000; // ms
const int MOTOR_PWM_ENABLE = 200;
const int BACKDOOR_PWM_UP = 110;
const int BACKDOOR_PWM_DOWN = 80;
const int BACKDOOR_TIMEOUT = 1000; // ms

// Don't use pin 13
IBT2 left = IBT2(12, 11);
IBT2 right = IBT2(10, 9);
L298N backdoor = L298N(8, 52, 53);
LimitSwitch ls_up = LimitSwitch(22);
LimitSwitch ls_down = LimitSwitch(23);

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
    backdoor.setup();

    ls_up.setup();
    ls_down.setup();

    Serial.begin(115200);
    Serial.setTimeout(0);

    Serial.println("hola");
}

void moveForward() {
    left.clockwise(MOTOR_PWM_ENABLE);
    right.clockwise(MOTOR_PWM_ENABLE);

    timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
        // measure distance with MPU6050
        return false;
    });

    left.stop();
    right.stop();
}

void moveBackward() {
    left.counterClockwise(MOTOR_PWM_ENABLE);
    right.counterClockwise(MOTOR_PWM_ENABLE);

    timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
        // measure distance with MPU6050
        return false;
    });

    left.stop();
    right.stop();
}

void openBackdoor() {
    backdoor.clockwise(BACKDOOR_PWM_UP);
    timeout_repeat(BACKDOOR_TIMEOUT, []() {
        return ls_up.read() == LOW;
    });
    backdoor.stop();
}

void closeBackdoor() {
    backdoor.counterClockwise(BACKDOOR_PWM_DOWN);
    timeout_repeat(BACKDOOR_TIMEOUT, []() {
        return ls_down.read() == LOW;
    });
    backdoor.stop();
}

void loop() {
    while (Serial.available() == 0) { ; }

    char instr = Serial.read();
    switch (instr) {
        case MoveForward:
            moveForward();
            break;

        case MoveBack:
            moveBackward();
            break;

        case DumpCans:
            openBackdoor();
            delay(1500);
            closeBackdoor();
            break;

        default:
            break;
    }

    Serial.println("aksjdfhas");
}
