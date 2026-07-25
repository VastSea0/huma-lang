const values = [];
for (let i = 0; i < 50_000; i += 1) {
  values.push(i % 997);
}
let total = 0;
for (const value of values) {
  total += value;
}
console.log(total);
