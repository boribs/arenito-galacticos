void setup() {
  Serial.begin(115200);
  Serial.setTimeout(0);

  Serial.println("<Arduino is ready>");
}

void loop() {
  while (Serial.available() == 0) {
    ;
  }

  String r = Serial.readString();
  if (r == "h") {
    Serial.println("hola");
  }
}
