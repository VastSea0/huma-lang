var values: [UInt64] = []
values.reserveCapacity(50_000)
for i in 0..<50_000 {
    values.append(UInt64(i % 997))
}
var total: UInt64 = 0
for value in values {
    total += value
}
print(total)
