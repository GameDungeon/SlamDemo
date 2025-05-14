import serial
import time

lidar_serial = serial.Serial("/dev/ttyACM0", baudrate=230400)
# odom_serial = serial.Serial("/dev/ttyACM1", baudrate=115200)

time.sleep(1)
lidar_serial.write(b"b")
print("Connected")

pos = [2, 2, 2]
start_pos = [0, 0, 0]

last_scan = 0

scans = []

with open("scans.data", "w") as f:
    f.write("New Program\n")
    while True:
        try:
            if lidar_serial.in_waiting:
                scan = str(lidar_serial.readline())[2:-3].split(",")

                if len(scan) != 2:
                    break

                if len(scans) == 360:
                    x = (pos[0] + start_pos[0]) / 2
                    y = (pos[1] + start_pos[1]) / 2
                    rot = (pos[2] + start_pos[2]) / 2
                    out = f"lidar {x} {y} {rot} "

                    for scan in scans:
                        out += f"{scan[0]} {scan[1]} "

                    out += "\n"

                    # print(out)
                    f.write(out)

                    scans = []
                    start_pos = pos

                scans.append(scan)

            # if odom_serial.in_waiting:
            #     line = str(odom_serial.readline())[2:-3]

            #     if line.startswith("odom"):
            #         new_pos = line.split(" ")[1:]
            #         pos = [new_pos[0], new_pos[1], new_pos[2]]
            #         print(pos)

        except (ValueError, IndexError) as e:
            print(e)
            continue
        except KeyboardInterrupt:
            lidar_serial.write(b"e")
            lidar_serial.close()
            print("Connection Closed")
            break
