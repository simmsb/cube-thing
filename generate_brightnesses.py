def generate():
    res = []
    for i in range(16, -1, -1):
        s = "1" + "0" * i
        s *= 33 // (i + 1)
        s = s.ljust(32, "0")
        s = s[:32]
        res.append(int(s, base=2))
    return res

brightnesses = generate()
for brightness in brightnesses:
    print(hex(brightness))
