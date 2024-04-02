class DCMotor {
    public:
    int in1, in2;

    DCMotor(int in1, int in2) {
        this->in1 = in1;
        this->in2 = in2;
    }

    void setup() {
        pinMode(this->in1, OUTPUT);
        pinMode(this->in2, OUTPUT);

        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);
    }

    void clockwise() {
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, HIGH);
    }

    void counterClockwise() {
        digitalWrite(this->in1, HIGH);
        digitalWrite(this->in2, LOW);
    }

    void stop() {
        digitalWrite(this->in1, LOW);
        digitalWrite(this->in2, LOW);
    }
};

DCMotor tapa = DCMotor(7, 6);

void setup() {
  Serial.begin(9600);
  tapa.setup();
}

void loop() {
  tapa.clockwise();
  Serial.println("Giro del Motor en sentido horario");
  delay(5000);

  tapa.counterClockwise();
  Serial.println("Giro del Motor en sentido antihorario");
  delay(5000);

  tapa.stop();
  Serial.println("Motor Detenido");
  delay(3000);
}
