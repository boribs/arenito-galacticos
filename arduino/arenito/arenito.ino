// Usando puentes H de los rojos
const int motDa = 6;
const int motDb = 7;
const int motIa = 8;
const int motIb = 9;

// Rodillo!
const int rodA = 4;
const int rodB = 3;

// sensor de proximidad, gracias Jaliscos
const int lr = 14;

void setup() {
  pinMode(motIa, OUTPUT);
  pinMode(motIb, OUTPUT);
  pinMode(motDa, OUTPUT);
  pinMode(motDb, OUTPUT);

  pinMode(rodA, OUTPUT);
  pinMode(rodB, OUTPUT);

  pinMode(lr, INPUT);

  Serial.begin(115200);
  Serial.setTimeout(50); // hay que checar esto
  Serial.print("ok");
}

void loop() {
  prendeRodillo();

  if (Serial.available() > 0) {
    char c = Serial.read();

    switch (c) {
      case 'a':
        avanza(100);
        break;
      case 'i':
        izquierda(50);
        break;
      case 'd':
        derecha(50);
        break;
      case 'r':
        retrocede(100);
        break;
      case 'l': // desuso
        derecha(800);
        break;
    }

    Serial.print('k');

    // if (leeUS(t1, e1) < MIN_DIST || leeUS(t2, e2) < MIN_DIST) {
    if (digitalRead(lr) == 0) {
      retrocede(1000);
      derecha(2000);
    }
  }
}

void prendeRodillo() {
  digitalWrite(rodA, HIGH);
  digitalWrite(rodB, LOW);
}

void apagaRodillo() {
  digitalWrite(rodA, LOW);
  digitalWrite(rodB, LOW);
}

void avanza(int tiempo) {
  digitalWrite(motIa, HIGH);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, HIGH);
  digitalWrite(motDb, LOW);
  delay(tiempo);
}

void retrocede(int tiempo) {
  digitalWrite(motIa, LOW);
  digitalWrite(motIb, HIGH);
  digitalWrite(motDa, LOW);
  digitalWrite(motDb, HIGH);
  delay(tiempo);
}

void derecha(int tiempo) {
  digitalWrite(motIa, HIGH);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, LOW);
  digitalWrite(motDb, HIGH);
  delay(tiempo);
}

void izquierda(int tiempo) {
  digitalWrite(motIa, LOW);
  digitalWrite(motIb, HIGH);
  digitalWrite(motDa, HIGH);
  digitalWrite(motDb, LOW);
  delay(tiempo);
}

void alto(int tiempo) {
  digitalWrite(motIa, LOW);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, LOW);
  digitalWrite(motDb, LOW);
  delay(tiempo);
}
