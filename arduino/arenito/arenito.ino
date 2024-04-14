#include "ArenitoUtils.h"

const int INSTRUCTION_EXECUTION_TIME = 200; // ms
const int MOTOR_MOVE_TIME = 200; // ms
const int MOTOR_PWM_ENABLE = 150;
const int MOTOR_ROT_PWM_ENABLE = 100;
const int BACKDOOR_PWM_UP = 180;
const int BACKDOOR_PWM_DOWN = 80;
const int BACKDOOR_TIMEOUT = 1000; // ms
const int BACKDOOR_EXT_PWM_ENABLE = 130;
const int BACKDOOR_EXT_TIME_UP = 2000; // ms
const int BACKDOOR_EXT_TIME_DOWN = 1600; // ms
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
// ultrasonic front/rear left/right
Ultrasonic ufl = Ultrasonic(24, 25);
Ultrasonic ufr = Ultrasonic(26, 27);
Ultrasonic url = Ultrasonic(28, 29);
Ultrasonic urr = Ultrasonic(30, 31);

IrProx irfl = IrProx(33);
IrProx irfr = IrProx(32);
IrProx irrl = IrProx(35);
IrProx irrr = IrProx(34);

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
    StopAll = 'S',
};

void setup() {
    left.setup();
    right.setup();
    backdoor.setup();
    brush.setup();

    ls_up.setup();
    ls_down.setup();

    ufl.setup();
    ufr.setup();
    url.setup();
    urr.setup();

    irfl.setup();
    irfr.setup();
    irrl.setup();
    irrr.setup();

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
    left.clockwise(MOTOR_ROT_PWM_ENABLE);
    right.counterClockwise(MOTOR_ROT_PWM_ENABLE);

    timeout_repeat(time, []() {
        return false;
    });

    // left.stop();
    // right.stop();
}

void moveRight(const int time) {
    left.counterClockwise(MOTOR_ROT_PWM_ENABLE);
    right.clockwise(MOTOR_ROT_PWM_ENABLE);

    timeout_repeat(time, []() {
        return false;
    });

    // left.stop();
    // right.stop();
}

void extendBackdoor() {
    backdoor_ext.clockwise(BACKDOOR_EXT_PWM_ENABLE);
    timeout_repeat(BACKDOOR_EXT_TIME_UP, []() {
        return false;
    });

    backdoor_ext.counterClockwise(BACKDOOR_EXT_PWM_ENABLE);
    timeout_repeat(BACKDOOR_EXT_TIME_DOWN, []() {
        return false;
    });
    backdoor_ext.stop();
}

void stopAll() {
    left.stop();
    right.stop();
    backdoor.stop();
    brush_on = false;
}

void loop() {
    while (Serial.available() == 0) {
        brush.clockwise(brush_on ?  BRUSH_PWM_ENABLE : 0);
    }

    char instr = Serial.read();
    switch (instr) {
        case InstructionMap::MoveForward:
            moveForward();
            break;

        case InstructionMap::MoveBack:
            moveBackward();
            break;

        case InstructionMap::MoveLeft:
            moveLeft(MOTOR_MOVE_TIME);
            break;

        case InstructionMap::MoveRight:
            moveRight(MOTOR_MOVE_TIME);
            break;

        case InstructionMap::MoveLongRight:
            moveRight(MOTOR_MOVE_TIME * 4);
            break;

        case InstructionMap::DumpCans:
            openBackdoor();
            delay(1500);
            closeBackdoor();
            break;

        case InstructionMap::RequestProxSensor:
            Serial.println(
                String(ufl.read()) + "," +
                String(ufr.read()) + "," +
                String(url.read()) + "," +
                String(urr.read()) + "," +
                String(irfl.inRange()) + "," +
                String(irfr.inRange()) + "," +
                String(irrl.inRange()) + "," +
                String(irrr.inRange())
            );
            break;

        case InstructionMap::BrushOn:
            brush_on = true;
            break;

        case InstructionMap::BrushOff:
            brush_on = false;
            break;

        case InstructionMap::ExtendBackdoor:
            extendBackdoor();
            break;

        case InstructionMap::StopAll:
            stopAll();
            break;

        default:
            break;
    }

    Serial.println("ok");
}
