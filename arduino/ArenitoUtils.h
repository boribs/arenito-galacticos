#ifndef __H_ARENITO_UTILS
#define __H_ARENITO_UTILS 1

class DCMotor {
    public:
    int in1, in2;

    DCMotor(int in1, int in2) {
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

    void counterClockwise() {
        digitalWrite(this->in1, HIGH);
        digitalWrite(this->in2, LOW);
    }

    void stop() {
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);
    }
};

#endif
