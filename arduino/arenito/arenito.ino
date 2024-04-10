#include "ArenitoUtils.h"

const int INSTRUCTION_EXECUTION_TIME = 200; // ms
const int MOTOR_MOVE_TIME = 200; // ms
const int MOTOR_PWM_ENABLE = 150;
const int MOTOR_ROT_PWM_ENABLE = 100;
const int BACKDOOR_PWM_UP = 180;
const int BACKDOOR_PWM_DOWN = 100;
const int BACKDOOR_TIMEOUT = 1000; // ms
const int BACKDOOR_EXT_PWM_ENABLE = 255;
const int BACKDOOR_EXT_TIME = 4000; // ms
const int BRUSH_PWM_ENABLE = 185;

// Don't use pin 13
IBT2 left = IBT2(12, 11);
IBT2 right = IBT2(10, 9);
L298N backdoor = L298N(8, 52, 53);
L298N brush = L298N(7, 50, 51);
L298N backdoor_ext = L298N(6, 48, 49);
LimitSwitch ls_up = LimitSwitch(22);
LimitSwitch ls_down = LimitSwitch(23);

//                    trigger, echo
Ultrasonic u1 = Ultrasonic(24, 25);
Ultrasonic u2 = Ultrasonic(26, 27);
Ultrasonic u3 = Ultrasonic(28, 29);
Ultrasonic u4 = Ultrasonic(30, 31);

bool brush_on = false;

enum InstructionMap {
    MoveForward = 'a',
    MoveLeft = 'i',
    MoveRight = 'd',
    MoveBack = 'r',
    MoveLongRight = 'D',
    RequestProxSensor = 's',
    DumpCans = 'c',
    BrushOn = 'P',
    BrushOff = 'p',
    ExtendBackdoor = 'e',
};

void setup() {
    left.setup();
    right.setup();
    backdoor.setup();
    brush.setup();

    ls_up.setup();
    ls_down.setup();

    u1.setup();
    u2.setup();
    u3.setup();
    u4.setup();

    Serial.begin(115200);
    Serial.setTimeout(0);

    Serial.println("Arduino listo");
}

void moveForward() {
    left.clockwise(MOTOR_PWM_ENABLE);
    right.clockwise(MOTOR_PWM_ENABLE);

    timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
        // measure distance with MPU6050
        return false;
    });

    // left.stop();
    // right.stop();
}

void moveBackward() {
    left.counterClockwise(MOTOR_PWM_ENABLE);
    right.counterClockwise(MOTOR_PWM_ENABLE);

    timeout_repeat(INSTRUCTION_EXECUTION_TIME, []() {
        // measure distance with MPU6050
        return false;
    });

    // left.stop();
    // right.stop();
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

void moveLeft(const int time) {
    left.clockwise(MOTOR_PWM_ENABLE);
    right.counterClockwise(MOTOR_PWM_ENABLE);

    timeout_repeat(time, []() {
        return false;
    });

    // left.stop();
    // right.stop();
}

void moveRight(const int time) {
    left.counterClockwise(MOTOR_PWM_ENABLE);
    right.clockwise(MOTOR_PWM_ENABLE);

    timeout_repeat(time, []() {
        return false;
    });

    // left.stop();
    // right.stop();
}

void extendBackdoor() {
    backdoor_ext.clockwise(BACKDOOR_EXT_PWM_ENABLE);
    timeout_repeat(BACKDOOR_EXT_TIME, []() {
        return false;
    });

    backdoor_ext.counterClockwise(BACKDOOR_EXT_PWM_ENABLE);
    timeout_repeat(BACKDOOR_EXT_TIME, []() {
        return false;
    });
    backdoor_ext.stop();
}

void loop() {
    while (Serial.available() == 0) {
        brush.clockwise(brush_on ?  BRUSH_PWM_ENABLE : 0);
    }

    char instr = Serial.read();
    switch (instr) {
        case MoveForward:
            moveForward();
            break;

        case MoveBack:
            moveBackward();
            break;

        case MoveLeft:
            moveLeft(MOTOR_MOVE_TIME);
            break;

        case MoveRight:
            moveRight(MOTOR_MOVE_TIME);
            break;

        case DumpCans:
            openBackdoor();
            delay(1500);
            closeBackdoor();
            break;

        case RequestProxSensor:
            Serial.println(
                String(u1.filterRead()) + "," +
                String(u2.filterRead()) + "," +
                String(u3.filterRead()) + "," +
                String(u4.filterRead())
            );
            break;

        case BrushOn:
            brush_on = true;
            break;

        case BrushOff:
            brush_on = false;
            break;

        case ExtendBackdoor:
            extendBackdoor();
            break;

        default:
            break;
    }

    Serial.println("ok");
}
