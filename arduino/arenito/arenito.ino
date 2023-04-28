// Usando puentes H de los rojos
const int motDa = 6;
const int motDb = 7;
const int motIa = 8;
const int motIb = 9;

// Rodillo!
const int rodA = 4;
const int rodB = 3;

// sensores ultrasónicos
const int t1 = 10;
const int e1 = 11;
const int t2 = 12;
const int e2 = 13;
const int t3 = 26;
const int e3 = 27;

const int MIN_DIST = 30;

void setup() {
  pinMode(motIa, OUTPUT);
  pinMode(motIb, OUTPUT);
  pinMode(motDa, OUTPUT);
  pinMode(motDb, OUTPUT);

  pinMode(rodA, OUTPUT);
  pinMode(rodB, OUTPUT);

  pinMode(t1, OUTPUT);
  pinMode(t2, OUTPUT);
  pinMode(t3, OUTPUT);
  pinMode(e1, INPUT);
  pinMode(e2, INPUT);
  pinMode(e3, INPUT);

  Serial.begin(115200);
  Serial.setTimeout(100); // hay que checar esto
  Serial.print("ok");
}

void loop() {
  prendeRodillo();

  if (Serial.available() > 0) {

    char c = Serial.read();

    if (leeUS(t1, e1) < MIN_DIST
        // leeUS(t2, e2) < MIN_DIST ||
        // leeUS(t3, e3) < MIN_DIST
    ) {
      retrocede(1000);
      derecha(2000);
      c = '/';
    }

    switch (c) {
      case 'a':
        avanza(100);
        break;
      case 'i':
        izquierda(100);
        break;
      case 'd':
        derecha(100);
        break;
      case 'r':
        retrocede(100);
        break;
      case 'l':
        derecha(800);
        break;
    }

    Serial.print('k');
  }
}

int ms2cm(long microsegundos) {
  return (int)(microsegundos / 29 / 2);
}

int leeUS(int trigger, int echo) {
  digitalWrite(trigger, LOW);
  delayMicroseconds(2);
  digitalWrite(trigger, HIGH);
  delayMicroseconds(10);
  digitalWrite(trigger, LOW);

  long duration = pulseIn(echo, HIGH);
  return ms2cm(duration);
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
