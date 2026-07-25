values = []
for i in range(50_000):
    values.append(i % 997)
total = 0
for value in values:
    total += value
print(total)
