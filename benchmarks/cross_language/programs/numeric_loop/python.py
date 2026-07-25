state = 1
total = 0
for _ in range(1, 200_001):
    state = (state * 1_664_525 + 1_013_904_223) % 4_294_967_296
    total = (total + state) % 9_007_199_254_740_881
print(total)
