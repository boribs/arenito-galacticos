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

  Serial.begin(115200);
  Serial.setTimeout(1000); // hay que checar esto
}

void loop() {
  if (Serial.available()) {

    // recuerda sobreponer el sensor ultrasónico

    String msg = Serial.readString();
    char c = msg[0];

    switch (c) {
      case 'a':
        avanza(10);
        break;
      case 'i':
        izquierda(10);
        break;
      case 'd':
        derecha(10);
        break;
      case 'r':
        retrocede(10);
        break;
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

/*
 * Pixel a MS(G - giro)
 * Se usa para determinar cuánto hay que girar
 * tomando en cuenta la distancia de la detección
 * al centro de la pantalla.
 *
 * minT y maxT son valores arbitrarios.
 */
int pxAMsG(float t) {
  const int minT = 200,
            maxT = 500;
  return lerp(minT, maxT, t);
}

/*
 * Pixel A Ms (A - Avanza)
 */
int pxAMsA(float t) {
  const int minT = 1000,
            maxT = 2000;
  return lerp(minT, maxT, t);
}

int lerp(int a, int b, float t) {
  return (int)(a + ((b - a) * t));
}
