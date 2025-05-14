import sys
import time
import serial
from PyQt6.QtWidgets import QApplication
import pyqtgraph as pg
import numpy as np

x = np.zeros(1000)
y = np.zeros(1000)

app = QApplication([])
plot = pg.plot(pen=None, symbol="o")
plot_curve = plot.plot(pen="blue", symbol="o")

with serial.Serial("/dev/ttyACM0", baudrate=230400) as ser:
    time.sleep(1)
    start_time = time.time()
    samples = 0
    ser.write(b"b")
    print("Connected")
    try:
        while True:
            line = ser.readline()
            try:
                degrees, new_x, new_y = tuple(str(line)[2:-5].split(","))
                samples += 1

                # degrees = round(float(degrees))
                # x[degrees] = new_x
                # y[degrees] = new_y

                x = np.roll(x, -1)
                x[-1] = new_x

                y = np.roll(y, -1)
                y[-1] = new_y

            except (ValueError, IndexError):
                print(line)
                continue

            if samples % 360 == 0:
                print(samples / (time.time() - start_time))

            # print(f"({new_x}, {new_y})")

            plot_curve.setData(np.array(x, dtype=float), np.array(y, dtype=float))

            QApplication.processEvents()
    finally:
        ser.write(b"e")
        print("Connection Close")
