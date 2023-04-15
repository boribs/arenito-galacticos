/*Funciones básicas para el uso de los motores
avanza, izquierda, derecha, alto*/
//Usando puentes H de los rojos

int motIb = 9;
int motIa = 8;
int motDb = 7;
int motDa = 6;

/*
 * Asume inicialmente una resolución de RES_X,RES_Ypx
 * y un margen de MARGENpx.
 * El margen es para la alineación con la lata seleccionada.
 */
const int RES_X = 640,
          RES_Y = 380,
          MARGEN = 15, // Determinar margen
          CENTRO_X_MIN = (RES_X / 2) - MARGEN,
          CENTRO_X_MAX = (RES_X / 2) + MARGEN;

const float CENTRO_X = RES_X / 2;

typedef struct lata {
  int x;
  int y;
} lata;

const int MAX_LATAS = 20;
lata detectadas[MAX_LATAS] = { 0 };
int eleigda = -1; // -1 = no se ha elegido ninguna

float distAlCentro, t;
int d;

void setup() {
  pinMode(motIa, OUTPUT);
  pinMode(motIb, OUTPUT);
  pinMode(motDa, OUTPUT);
  pinMode(motDb, OUTPUT);

  Serial.begin(115200);
  Serial.setTimeout(100); // hay que checar esto
}

void loop() {
  if (Serial.available() > 0) {
    int n = descifraLatas();

    // PARA EL CASO DE PRUEBA ESPECIFICO
    // CUANDO ENCUENTRA SOLO UNA LATA
    if (n == 1) {
      Serial.print("Ok: ");
      int d = detectadas[0].x;
      // si la lata está centrada, camina hacia esta
      if (d >= CENTRO_X_MIN && d <= CENTRO_X_MAX) {
        // avanza(pxAMs(detectadas[0].y));
        alto(0);
      }

      // si la lata está a la izquierda del margen, gira derecha
      if (d < CENTRO_X_MIN) {
        distAlCentro = CENTRO_X - d;
        t = distAlCentro / CENTRO_X;

        derecha(pxAMs(t));
        alto(0);
      }

      // si la lata está a la derecha del margen, gira izquierda
      if (d > CENTRO_X_MAX) {
        distAlCentro = d - CENTRO_X;
        t = distAlCentro / CENTRO_X;

        izquierda(pxAMs(t));
        alto(0);
      }
    }
  }
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

void avanza(int tiempo) {
  digitalWrite(motIa, HIGH);
  digitalWrite(motIb, LOW);
  digitalWrite(motDa, HIGH);
  digitalWrite(motDb, LOW);
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
 * Pixel a MS.
 * Se usa para determinar cuánto hay que girar
 * tomando en cuenta la distancia de la detección
 * al centro de la pantalla.
 *
 * minT y maxT son valores arbitrarios.
 */
int pxAMs(float t) {
  const int minT = 200,
            maxT = 500;
  return lerp(minT, maxT, t);
}

int lerp(int a, int b, float t) {
  return (int)(a + ((b - a) * t));
}
