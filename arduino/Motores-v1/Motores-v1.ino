/*Funciones básicas para el uso de los motores
avanza, izquierda, derecha, alto*/
//Usando puentes H de los rojos

int motIb = 9;
int motIa = 8;
int motDb = 7;
int motDa = 6;

typedef struct lata {
  int x;
  int y;
} lata;

const int MAX_LATAS = 20;
lata detectadas[MAX_LATAS] = { 0 };
int eleigda = -1; // -1 = no se ha elegido ninguna

void setup() {
  pinMode(motIa, OUTPUT);
  pinMode(motIb, OUTPUT);
  pinMode(motDa, OUTPUT);
  pinMode(motDb, OUTPUT);
}

void loop() {
  
}

/*
 * Decodifica el mensaje enviado por el puerto serial
 * que contiene los puntos medios de las detecciones
 * de las latas.
 *
 * Ejemplo: {x1,y1,x2,y2,...,}
 *
 * Todos los valores son enteros positivos separados
 * por comas. El último valor también debe llevar una
 * coma al final (antes de }).
 *
 * Regresa el número de latas encontradas o -1 cuando
 * ocurrió algún error.
 */
int descifraLatas() {
  String msg = Serial.readString();
  msg.trim();
  size_t msg_len = msg.length();

  if (msg[0] != '{' || msg[msg_len - 1] != '}') {
    // Los delimitadores son incorrectos:
    // no empieza con { ni termina con }
    return -1;
  }

  int num_latas = 0;
  String num = "";
  bool set_x = false;

  for (int i = 0; i < msg_len - 1; ++i) {
    if (isDigit(msg[i])) {
      num += msg[i];
    } else if (msg[i] == ',') {
      if (!set_x) {
        detectadas[num_latas].x = num.toInt();
        num = "";
      } else {
        detectadas[num_latas].y = num.toInt();
        num = "";
        num_latas++;
      }
      set_x = !set_x;
    }
  }

  return num_latas;
}

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
