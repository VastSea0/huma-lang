func mix(_ state: UInt64, _ index: UInt64) -> UInt64 {
    (state * 48_271 + index) % 2_147_483_647
}

var state: UInt64 = 1
for i in 1...100_000 {
    state = mix(state, UInt64(i))
}
print(state)
