#include <Wire.h>

float RateRoll, RatePitch, RateYaw, RateCalibrationRoll, RateCalibrationPitch, RateCalibrationYaw;
float AccX, AccY, AccZ, AngleRoll, AnglePitch, VelocityX = 0, VelocityY = 0;
int RateCalibrationNumber;
float AngleYaw, KalmanAngleYaw, KalmanUncertaintyAngleYaw;
uint32_t LoopTimer;


float KalmanAngleRoll = 0, KalmanUncertaintyAngleRoll = 2 * 2, KalmanAnglePitch = 0, KalmanUncertaintyAnglePitch = 2 * 2;
float KalmanVelocityX = 0, KalmanUncertaintyVelocityX = 0.1 * 0.1;
float KalmanVelocityY = 0, KalmanUncertaintyVelocityY = 0.1 * 0.1;

float Kalman1DOutput[] = { 0, 0 };  //Prediccion valores

void kalman_1D(float KalmanState, float KalmanUncertainty, float KalmanInput, float KalmanMeasurement) {

  //KalmanInput/Taza de rotacion, KalmanMeasurement/Angulo acelerometro, KalmanState/Angulo calculado tras el filtro
  KalmanState = KalmanState + 0.004 * KalmanInput;
  //Predecir estado actual del sistema
  KalmanUncertainty = KalmanUncertainty + 0.004 * 0.004 * 4 * 4;
  //Calcular la incertidumbre de la prediccion
  float KalmanGain = KalmanUncertainty * 1 / (1 * KalmanUncertainty + 3 * 3);
  //Calcular la ganancia de la incertidumbre
  KalmanState = KalmanState + KalmanGain * (KalmanMeasurement - KalmanState);
  //Actualizar el estado
  KalmanUncertainty = (1 - KalmanGain) * KalmanUncertainty;

  Kalman1DOutput[0] = KalmanState;
  Kalman1DOutput[1] = KalmanUncertainty;
}

void accelerometerSignals(void) {
  Wire.beginTransmission(0x68);
  Wire.write(0x1A);
  Wire.write(0x05);
  Wire.endTransmission();
  Wire.beginTransmission(0x68);
  Wire.write(0x1C);
  Wire.write(0x10);
  Wire.endTransmission();
  Wire.beginTransmission(0x68);
  Wire.write(0x3B);
  Wire.endTransmission();
  Wire.requestFrom(0x68, 6);
  int16_t AccXLSB = Wire.read() << 8 | Wire.read();
  int16_t AccYLSB = Wire.read() << 8 | Wire.read();
  int16_t AccZLSB = Wire.read() << 8 | Wire.read();
  Wire.beginTransmission(0x68);
  Wire.write(0x1B);
  Wire.write(0x8);
  Wire.endTransmission();
  Wire.beginTransmission(0x68);
  Wire.write(0x43);
  Wire.endTransmission();
  Wire.requestFrom(0x68, 6);
  int16_t GyroX = Wire.read() << 8 | Wire.read();
  int16_t GyroY = Wire.read() << 8 | Wire.read();
  int16_t GyroZ = Wire.read() << 8 | Wire.read();

  RateRoll = (float)GyroX / 65.5;
  RatePitch = (float)GyroY / 65.5;
  RateYaw = (float)GyroZ / 65.5;

  AccX = (float)AccXLSB / 4096;
  AccY = (float)AccYLSB / 4096;
  AccZ = (float)AccZLSB / 4096;

  AngleRoll = atan(AccY / sqrt(AccX * AccX + AccZ * AccZ)) * 1 / (3.142 / 180);
  AnglePitch = -atan(AccX / sqrt(AccY * AccY + AccZ * AccZ)) * 1 / (3.142 / 180);

  VelocityX += AccX * 0.004;  // 0.004 segundos entre las muestras
  VelocityY += AccY * 0.004; 
}

void setup() {
  Serial.begin(9600);
  pinMode(13, OUTPUT);
  digitalWrite(13, HIGH);
  Wire.setClock(400000);
  Wire.begin();
  delay(250);
  Wire.beginTransmission(0x68);
  Wire.write(0x6B);
  Wire.write(0x00);
  Wire.endTransmission();

  for (RateCalibrationNumber = 0; RateCalibrationNumber < 2000; RateCalibrationNumber++) {
    accelerometerSignals();
    RateCalibrationRoll += RateRoll;
    RateCalibrationPitch += RatePitch;
    delay(1);
  }
  RateCalibrationRoll /= 2000;
  RateCalibrationPitch /= 2000;
  LoopTimer = micros();
}

void loop() {
  // put your main code here, to run repeatedly:
  accelerometerSignals();
  RateRoll -= RateCalibrationRoll;
  RatePitch -= RateCalibrationPitch;
  RateYaw -= RateCalibrationYaw;

  kalman_1D(KalmanAngleRoll, KalmanUncertaintyAngleRoll, RateRoll, AngleRoll);
  KalmanAngleRoll = Kalman1DOutput[0];
  KalmanUncertaintyAngleRoll = Kalman1DOutput[1];
  kalman_1D(KalmanAnglePitch, KalmanUncertaintyAnglePitch, RatePitch, AnglePitch);
  KalmanAnglePitch = Kalman1DOutput[0];
  KalmanUncertaintyAnglePitch = Kalman1DOutput[1];

  kalman_1D(KalmanAngleYaw, KalmanUncertaintyAngleYaw, RateYaw, AngleYaw);
  KalmanAngleYaw = Kalman1DOutput[0];
  KalmanUncertaintyAngleYaw = Kalman1DOutput[1];

  kalman_1D(KalmanVelocityX, KalmanUncertaintyVelocityX, AccX, VelocityX);
  KalmanVelocityX = Kalman1DOutput[0];
  KalmanUncertaintyVelocityX = Kalman1DOutput[1];

  kalman_1D(KalmanVelocityY, KalmanUncertaintyVelocityY, AccY, VelocityY);
  KalmanVelocityY = Kalman1DOutput[0];
  KalmanUncertaintyVelocityY = Kalman1DOutput[1];

  Serial.print("Roll Angle: ");
  Serial.println(KalmanAngleRoll);
  Serial.print("Pitch Angle: ");
  Serial.println(KalmanAnglePitch);
  Serial.print("Yaw Angle: ");
  Serial.println(KalmanAngleYaw);
  Serial.print("Velocity X: ");
  Serial.println(KalmanVelocityX);
  Serial.print("Velocity Y: ");
  Serial.println(KalmanVelocityY);

  while (micros() - LoopTimer < 4000)
    ;
  LoopTimer = micros();
}
