let state = 1;
let total = 0;
for (let i = 1; i <= 200_000; i += 1) {
  state = (state * 1_664_525 + 1_013_904_223) % 4_294_967_296;
  total = (total + state) % 9_007_199_254_740_881;
}
console.log(total.toFixed(0));
