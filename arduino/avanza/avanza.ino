// Usando puentes H de los rojos
int motDa = 6;
int motDb = 7;
int motIa = 8;
int motIb = 9;

// Rodillo!
int rodA = 4;
int rodB = 3;

void setup() {
    pinMode(motIa, OUTPUT);
    pinMode(motIb, OUTPUT);
    pinMode(motDa, OUTPUT);
    pinMode(motDb, OUTPUT);
    pinMode(rodA, OUTPUT);
    pinMode(rodB, OUTPUT);

    avanza(1000);
    alto(1000);
    derecha(1000);
    alto(1000);
    izquierda(1000);
    alto(1000);
    retrocede(100);
    alto(1000);
}

void loop() {

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
  prendeRodillo();
  digitalWrite(motIa, HIGH);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, HIGH);
  digitalWrite(motDb, LOW);
  delay(tiempo);
  apagaRodillo();
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
