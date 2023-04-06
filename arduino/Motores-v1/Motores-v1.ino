/*Funciones básicas para el uso de los motores
avanza, izquierda, derecha, alto*/
//Usando puentes H de los rojos

int motIb = 9;
int motIa = 8;
int motDb = 7;
int motDa = 6;

void setup() {
  pinMode(motIa, OUTPUT);
  pinMode(motIb, OUTPUT);
  pinMode(motDa, OUTPUT);
  pinMode(motDb, OUTPUT);
}

void loop() {
  
}

void avanza () {
  digitalWrite(motIa, HIGH);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, HIGH);
  digitalWrite(motDb, LOW);
}

void derecha () {
  digitalWrite(motIa, HIGH);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, LOW);
  digitalWrite(motDb, HIGH);
}

void izquierda () {
  digitalWrite(motIa, LOW);
  digitalWrite(motIb, HIGH);
  digitalWrite(motDa, HIGH);
  digitalWrite(motDb, LOW);
}

void alto () {
  digitalWrite(motIa, LOW);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, LOW);
  digitalWrite(motDb, LOW);
}
