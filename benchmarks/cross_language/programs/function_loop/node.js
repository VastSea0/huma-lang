function mix(state, index) {
  return (state * 48_271 + index) % 2_147_483_647;
}

let state = 1;
for (let i = 1; i <= 100_000; i += 1) {
  state = mix(state, i);
}
console.log(state.toFixed(0));
