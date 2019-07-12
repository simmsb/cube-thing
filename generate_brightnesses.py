def generate():
    res = []
    for i in range(32):
        s = "0" * (32 - i) + "1" * i
        res.append(int(s, base=2))
    return res

brightnesses = generate()
for brightness in brightnesses:
    print(f"0b{brightness:032b}")
