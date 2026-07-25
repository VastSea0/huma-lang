def mix(state, index):
    return (state * 48_271 + index) % 2_147_483_647


state = 1
for i in range(1, 100_001):
    state = mix(state, i)
print(state)
