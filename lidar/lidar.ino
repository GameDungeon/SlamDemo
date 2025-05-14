byte Frame [2520];
byte Sync[2];
int Ready = 0;

void setup() {
  pinMode(9, OUTPUT);
  pinMode(8, OUTPUT);

  Serial.begin(230400);
  Serial1.begin(230400);
  Serial1.write('e');
}

void loop() {
  if (Serial.available()) {
    Serial1.write(Serial.read());
  }

  //analogWrite(9, 100);
  //analogWrite(8, 100);
  
  //First capture the start byte of a frame:
  if (Serial1.available()) {
    Sync[0] = Serial1.read();
    if (Sync[0] == 0xFA) {
      while (!Serial1.available()) {
        Serial1.write(Serial.read());
      }
      Sync[1] = Serial1.read();
    }
  }  
  
  //Once start byte captured, read remaining frame into array:
 if (Sync[0] == 0xFA && Sync [1] == 0xA0) {
    Frame[0] = 0xFA;
    Frame[1] = 0xA0;
    for (int v = 2; v <= 2520; v++) {
      while (!Serial1.available()) {
        Serial1.write(Serial.read());
      }
      Frame[v] = Serial1.read();
    }
    Ready = 1;
  }  
  
  //Once frame captured, extract range/angle and convert to x/y:
  if (Ready == 1) {
    for (int i = 0; i < 2520; i = i + 42) {
      if (Frame[i] == 0xFA && Frame[i + 1] == 0xA0 + (i / 42)) {
        for (int j = i + 4; j < i + 40; j = j + 6) {
          int rangeA = Frame[j + 2];
          int rangeB = Frame[j + 3];
          int degrees = 6 * (i / 42) + (j - 4 - i) / 6;
          int range = (rangeB << 8) + rangeA;
          if (degrees != 0 && range != 0) {
            Serial.print(degrees);
            Serial.print(",");
            Serial.print(range);
            Serial.print("\n");
          }
        }
      }
    }
    Ready = 0;
  } else {
    Sync[0] = 0;
    Sync [1] = 0;
  }
}
