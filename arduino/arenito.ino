void setup() {
  Serial.begin(115200);
  Serial.setTimeout(1);
}

void loop() {

  // pide latas
  Serial.write("latas");
  while (!Serial.available());

  if (Serial.readString() == "ok") {
    // recibió latas
    // determina cómo avanzar
  }
}