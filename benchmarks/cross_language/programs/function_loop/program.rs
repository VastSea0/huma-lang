fn mix(state: u64, index: u64) -> u64 {
    (state * 48_271 + index) % 2_147_483_647
}

fn main() {
    let mut state: u64 = 1;
    for i in 1..=100_000 {
        state = mix(state, i);
    }
    println!("{state}");
}
