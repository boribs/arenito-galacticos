#ifndef __H_ARENITO_UTILS
#define __H_ARENITO_UTILS 1

#include <stdint.h>

const int PIN_UNSET = -1;

class DCMotor {
    public:
    int enable, in1, in2;

    DCMotor(int enable, int in1, int in2) {
        this->enable = enable;
        this->in1 = in1;
        this->in2 = in2;
    }

    DCMotor(int in1, int in2) {
        this->enable = PIN_UNSET;
        this->in1 = in1;
        this->in2 = in2;
    }

    void setup() {
        pinMode(this->in1, OUTPUT);
        pinMode(this->in2, OUTPUT);

        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);
    }

    void clockwise() {
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, HIGH);
    }

    void clockwise(uint8_t enable) {
        analogWrite(this->enable, enable);
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, HIGH);
    }

    void counterClockwise() {
        digitalWrite(this->in1, HIGH);
        digitalWrite(this->in2, LOW);
    }

    void counterClockwise(uint8_t enable) {
        analogWrite(this->enable, enable);
        digitalWrite(this->in1, HIGH);
        digitalWrite(this->in2, LOW);
    }

    void stop() {
        digitalWrite(this->enable, 0);
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);
    }
};

#endif
