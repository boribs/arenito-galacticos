#ifndef __H_ARENITO_UTILS
#define __H_ARENITO_UTILS 1

const int PIN_UNSET = -1;

typedef bool (*bool_func)();
typedef unsigned long ulong_t;

class DCMotor {
    public:
    int enable, in1, in2;

    /*
     * Sets motor pins, expected H-bridge.
     */
    DCMotor(int enable, int in1, int in2) {
        this->enable = enable;
        this->in1 = in1;
        this->in2 = in2;
    }

    /*
     * Sets motor pins.
     */
    DCMotor(int in1, int in2) {
        this->enable = PIN_UNSET;
        this->in1 = in1;
        this->in2 = in2;
    }

    /*
     * Configures `in1` and `in2` ports. Sets them both to LOW;
     * Same for `enable`, if set.
     */
    void setup() {
        pinMode(this->in1, OUTPUT);
        pinMode(this->in2, OUTPUT);

        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);

        if (this->enable != PIN_UNSET) {
            pinMode(this->enable, OUTPUT);
        }
    }

    /*
     * Clockwise movement, full speed.
     */
    void clockwise() {
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, HIGH);
    }

    /*
     * Clockwise movement, speed relative to `enable` pin.
     * `enable` must be in the 0-255 range.
     */
    void clockwise(uint8_t enable) {
        analogWrite(this->enable, enable);
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, HIGH);
    }

    /*
     * Counterclockwise movement, full speed.
     */
    void counterClockwise() {
        digitalWrite(this->in1, HIGH);
        digitalWrite(this->in2, LOW);
    }

    /*
     * Counterclockwise movement, speed relative to `enable` pin.
     * `enable` must be in the 0-255 range.
     */
    void counterClockwise(uint8_t enable) {
        analogWrite(this->enable, enable);
        digitalWrite(this->in1, HIGH);
        digitalWrite(this->in2, LOW);
    }

    /*
     * Stops motor.
     */
    void stop() {
        if (this->enable != PIN_UNSET) {
            analogWrite(this->enable, 0);
        }

        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);
    }
};

/*
 * Repeat until either `bool_func` is done or `timeout_ms` is reached.
 * `bool_func` is expected to return true when done executing.
 */
void timeout_repeat(ulong_t timeout_ms, bool_func callback) {
    ulong_t time = millis();

    while (millis() - time < timeout_ms) {
        if (callback()) {
            break;
        }
    }
}

#endif
