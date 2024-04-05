#include "ArenitoUtils.h"

const int INSTRUCTION_EXECUTION_TIME = 100; // ms
const int MOTOR_PWM_ENABLE = 200;
const int BACKDOOR_PWM_UP = 130;
const int BACKDOOR_PWM_DOWN = 80;
const int BACKDOOR_TIMEOUT = 1000; // ms

IBT2 left = IBT2(13, 12);
IBT2 right = IBT2(11, 10);
L298N backdoor = L298N(9, 22, 23);
LimitSwitch ls_up = LimitSwitch(53);
LimitSwitch ls_down = LimitSwitch(52);

void setup() {
    Serial.begin(115200);
    Serial.setTimeout(0);

    left.setup();
    right.setup();
    backdoor.setup();

    Serial.println("Arduino ready");
}

enum InstructionMap {
    MoveForward = 'a',
    MoveLeft = 'i',
    MoveRight = 'd',
    MoveBack = 'r',
    MoveLongRight = 'D',
    RequestProxSensor = 's',
    DumpCans = 'c',
};

void loop() {
    while (Serial.available() == 0) { ; }

    char instr = Serial.readString()[0];
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

        case MoveLeft:
            left.clockwise(MOTOR_PWM_ENABLE);
            right.counterClockwise(MOTOR_PWM_ENABLE);

            timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
                // measure distance with MPU6050
            });

            left.stop();
            right.stop();
            break;

        case MoveRight:
            left.counterClockwise(MOTOR_PWM_ENABLE);
            right.clockwise(MOTOR_PWM_ENABLE);

            timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
                // measure distance with MPU6050
            });

            left.stop();
            right.stop();
            break;

        case MoveBack:
            left.counterClockwise(MOTOR_PWM_ENABLE);
            right.counterClockwise(MOTOR_PWM_ENABLE);

            timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
                // measure distance with MPU6050
            });

            left.stop();
            right.stop();
            break;

        case MoveLongRight:
            left.counterClockwise(MOTOR_PWM_ENABLE);
            right.clockwise(MOTOR_PWM_ENABLE);

            timeout_repeat(3 * INSTRUCTION_EXECUTION_TIME, []() {
                // measure distance with MPU6050
            });

            left.stop();
            right.stop();
            break;

        case RequestProxSensor:
            // TODO: Finish this.
            Serial.print("25,25,100,100,");
            break;

        case DumpCans:
            // when it reaches this point
            // its already aligned with deposit

            // open backdoor
            backdoor.clockwise(BACKDOOR_PWM_UP);
            timeout_repeat(BACKDOOR_TIMEOUT, []() {
                return ls_up.read() == LOW;
            });
            backdoor.stop();

            // delay?
            // wiggle?

            // close backdoor
            backdoor.counterClockwise(BACKDOOR_PWM_DOWN);
            timeout_repeat(BACKDOOR_TIMEOUT, []() {
                return ls_down.read() == LOW;
            });
            backdoor.stop();
            break;

        default:
            break;
    }

    // confirmation message
    Serial.println("ok");
}
