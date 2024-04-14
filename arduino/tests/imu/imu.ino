#include "ArenitoUtils.h"

MPU6050 mpu;
unsigned long t, nt;

void setup(void) {
    Serial.begin(115200);
    mpu.setup();

    t = millis();
}

void loop() {
    mpu.read();
    nt = millis();
    Serial.println(
        String(nt - t) + ": " +
        String(mpu.acc().x) + "," +
        String(mpu.acc().y) + "," +
        String(mpu.acc().z)
    );
    t = nt;
}
