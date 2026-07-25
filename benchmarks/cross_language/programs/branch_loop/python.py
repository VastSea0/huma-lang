state = 1
total = 0
for i in range(1, 200_001):
    state = state * 2 + i
    if state >= 1_000_000_000:
        state -= 1_000_000_000
    if state >= 1_000_000_000:
        state -= 1_000_000_000
    total += state
print(total)
