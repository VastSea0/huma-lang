let state = 1;
let total = 0;
for (let i = 1; i <= 200_000; i += 1) {
  state = state * 2 + i;
  if (state >= 1_000_000_000) {
    state -= 1_000_000_000;
  }
  if (state >= 1_000_000_000) {
    state -= 1_000_000_000;
  }
  total += state;
}
console.log(total.toFixed(0));
