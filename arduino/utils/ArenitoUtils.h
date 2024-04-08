#ifndef __H_ARENITO_UTILS
#define __H_ARENITO_UTILS 1

#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>
#include <Wire.h>
// #include "quickVec3.h"

typedef bool (*bool_func)();
typedef unsigned long ulong_t;

const int PIN_UNSET = -1;
const ulong_t PULSE_IN_TIMEOUT = 5000;

ulong_t filterArray[10]; // array to store data samples from sensor

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
        analogWrite(this->forward, 0);
        analogWrite(this->backward, 0);
    }
};

class Ultrasonic {
    private:
    ulong_t lastDuration = 0,
            maxDuration = 11650, // around 200cm
            maxRange = 100; // no idea what this is for

    float noiseReject = 0.25; // percentage

    public:
    int echo, trigger;
    static const float SPEED_OF_SOUND = 29.1; // µs/cm

    /*
     * Sets echo and trigger pins.
     */
    Ultrasonic(int trigger, int echo) {
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

    ulong_t filterRead() {
        // 1. TAKING MULTIPLE MEASUREMENTS AND STORE IN AN ARRAY
        for (int sample = 0; sample < 10; sample++) {
            filterArray[sample] = read();
            delay(5); // to avoid untrasonic interfering
        }

        // 2. SORTING THE ARRAY IN ASCENDING ORDER
        for (int i = 0; i < 10; i++) {
            for (int j = i + 1; j < 10; j++) {
                if (filterArray[i] > filterArray[j]) {
                    ulong_t swap = filterArray[i];
                    filterArray[i] = filterArray[j];
                    filterArray[j] = swap;
                }
            }
        }

        // 3. FILTERING NOISE
        // + the five smallest samples are considered as noise -> ignore it
        // + the five biggest  samples are considered as noise -> ignore it
        // ----------------------------------------------------------------
        // => get average of the 10 middle samples (from 5th to 14th)
        ulong_t sum = 0;
        for (int sample = 2; sample < 8; sample++) {
            sum += filterArray[sample];
        }

        return sum / 6;
    }

    /*
     * Returns the distance in cm readout from this sensor.
     */
    ulong_t read() {
        // I don't love this.
        digitalWrite(this->trigger, LOW);
        delayMicroseconds(2);
        digitalWrite(this->trigger, HIGH);
        delayMicroseconds(10);
        digitalWrite(this->trigger, LOW);

        ulong_t duration = pulseIn(this->echo, HIGH, PULSE_IN_TIMEOUT);
        ulong_t unfiltered = (duration / 2) / SPEED_OF_SOUND;

        // https://github.com/MrNerdy404/HC-SR04_Filter/blob/master/SR04_Filter.ino
        if (duration <= 8) duration = ((this->maxRange + 1) * SPEED_OF_SOUND * 2);
        if (this->lastDuration == 0) this->lastDuration = duration;
        if (duration > (5 * this->maxDuration)) duration = this->lastDuration;
        if (duration > this->maxDuration) duration = this->maxDuration;

        if ((duration - this->lastDuration) < (-1.0 * this->noiseReject * this->lastDuration)){
            return (this->lastDuration / 2) / SPEED_OF_SOUND;
        }

        this->lastDuration = duration;
        return (duration / 2) / SPEED_OF_SOUND;
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
class MPU6050 {
    public:
    Adafruit_MPU6050 mpu;
    sensors_event_t a, g, t;
    Vec3 cal_a;

    MPU6050() {}

    void setup() {
        mpu.begin();
        mpu.setAccelerometerRange(MPU6050_RANGE_2_G);
        mpu.setGyroRange(MPU6050_RANGE_500_DEG);
        mpu.setFilterBandwidth(MPU6050_BAND_5_HZ);

        calibrate();

        // Serial.println(
        //     String(cal_a.x) + "," +
        //     String(cal_a.y) + "," +
        //     String(cal_a.z) + "A"
        // );
    }

    void calibrate() {
        int numMuestras = 1000;

        for (int i = 0; i < numMuestras; i++) {
            read();
            cal_a.x += a.acceleration.x;
            cal_a.y += a.acceleration.y;
            cal_a.z += a.acceleration.z;
            delay(10);
        }

        cal_a = cal_a / numMuestras;
    }

    void read() {
        mpu.getEvent(&a, &g, &t);
    }

    Vec3 acc() {
        return Vec3(
            a.acceleration.x,
            a.acceleration.y,
            a.acceleration.z
        ) - cal_a;
    }
};
*/

/*
 * Repeat until either `bool_func` is done or `timeout_ms` is reached.
 * `bool_func` is expected to return true when done executing.
 */
static void timeout_repeat(ulong_t timeout_ms, bool_func stop_condition) {
    ulong_t time = millis();

    while (millis() - time < timeout_ms) {
        if (stop_condition()) {
            break;
        }
    }
}

#endif
