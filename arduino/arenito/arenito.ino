void setup() {
  Serial.begin(115200);
  Serial.setTimeout(0);

  Serial.println("Arduino ready");
}

enum InstructionMap {
  MoveForward = 'a',
  MoveLeft = 'i',
  MoveRight = 'd',
  MoveBack = 'r',
  MoveLongRight = 'D',
  RequestProxSensor = 's',
};

void loop() {
  while (Serial.available() == 0) { ; }

  char instr = Serial.readString()[0];
  switch (instr) {
    case MoveForward:
      break;
    case MoveLeft:
      break;
    case MoveRight:
      break;
    case MoveBack:
      break;
    case MoveLongRight:
      break;
    case RequestProxSensor:
      // TODO: Finish this.
      Serial.println("25,25,100,100,");
      break;
    default:
      break;
  }

  Serial.println("ok");
}
