#ifndef __H_ARENITO_UTILS
#define __H_ARENITO_UTILS 1

const int PIN_UNSET = -1;

typedef bool (*bool_func)();
typedef unsigned long ulong_t;

/*
 * L297N H-bridge controller.
 */
class L298N { // single
    public:
    int enable, in1, in2;

    L298N(int enable, int in1, int in2) {
        this->enable = enable;
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
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);
    }
};

/*
 * IBT-2 H-bridge controller.
 */
class IBT2 {
    public:
    int forward, backward;

    IBT2(int forward, int backward) {
        this->forward = forward;
        this->backward = backward;
    }

    /*
     * Configures forward and backward pins.
     * Must be both PWM.
     */
    void setup() {
        pinMode(this->forward, OUTPUT);
        pinMode(this->backward, OUTPUT);
    }

    /*
     * Clockwise movement, speed relative to `enable` pin.
     * `enable` must be in the 0-255 range.
     */
    void clockwise(uint8_t speed) {
        analogWrite(this->forward, speed);
        analogWrite(this->backward, 0);
    }

    /*
     * Counterclockwise movement, speed relative to `enable` pin.
     * `enable` must be in the 0-255 range.
     */
    void counterClockwise(uint8_t speed) {
        analogWrite(this->forward, 0);
        analogWrite(this->backward, speed);
    }

    /*
     * Stops motor.
     */
    void stop() {
        digitalWrite(this->forward, LOW);
        digitalWrite(this->backward, LOW);
    }
};

class Ultrasonic {
    public:
    int echo, trigger;

    /*
     * Sets echo and trigger pins.
     */
    Ultrasonic(int echo, int trigger) {
        this->echo = echo;
        this->trigger = trigger;
    }

    /*
     * Configures `echo` and `trigger` pins.
     */
    void setup() {
        pinMode(this->echo, INPUT);
        pinMode(this->trigger, OUTPUT);
        digitalWrite(this->trigger, LOW);
    }

    /*
     * Returns the distance in cm readout from this sensor.
     */
    long read() {
        digitalWrite(this->trigger, HIGH);
        delayMicroseconds(10);
        digitalWrite(this->trigger, LOW);

        float duration = pulseIn(this->echo, HIGH);
        // https://arduinogetstarted.com/tutorials/arduino-ultrasonic-sensor
        // TODO: Filter noise.
        return duration * 0.017;
    }
};

class LimitSwitch {
    public:
    int pin;

    LimitSwitch(int pin) {
        this->pin = pin;
    }

    /*
     * Configures input pin.
     */
    void setup() {
        pinMode(this->pin, INPUT);
    }

    /*
     * Returns digital read of pin.
     */
    int read() {
        return digitalRead(this->pin);
    }
};

/*
 * Repeat until either `bool_func` is done or `timeout_ms` is reached.
 * `bool_func` is expected to return true when done executing.
 */
void timeout_repeat(ulong_t timeout_ms, bool_func stop_condition) {
    ulong_t time = millis();

    while (millis() - time < timeout_ms) {
        if (stop_condition()) {
            break;
        }
    }
}

#endif
