out = []
with open("intel.log", "r") as f:
    for line in f:
        if line.startswith("FLASER"):
            lineTokens = line.split()
            numPoints = int(lineTokens[1])
            range = lineTokens[2 : numPoints + 2]
            range = [float(r) for r in range]
            x, y, theta, gfsTimeStamp = (
                float(lineTokens[numPoints + 2]),
                float(lineTokens[numPoints + 3]),
                float(lineTokens[numPoints + 4]),
                float(lineTokens[numPoints + 8]),
            )
            out.append({"x": x, "y": y, "theta": theta, "range": range})

print(len(out))

with open("data.log", "w") as f:
    for x in out:
        f.write(str(x).replace("'", '"') + "\n")
