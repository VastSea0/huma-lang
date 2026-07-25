var state: UInt64 = 1
var total: UInt64 = 0
for i in 1...200_000 {
    state = state * 2 + UInt64(i)
    if state >= 1_000_000_000 {
        state -= 1_000_000_000
    }
    if state >= 1_000_000_000 {
        state -= 1_000_000_000
    }
    total += state
}
print(total)
