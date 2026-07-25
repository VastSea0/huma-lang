fn main() {
    let mut state: u64 = 1;
    let mut total: u64 = 0;
    for i in 1..=200_000 {
        state = state * 2 + i;
        if state >= 1_000_000_000 {
            state -= 1_000_000_000;
        }
        if state >= 1_000_000_000 {
            state -= 1_000_000_000;
        }
        total += state;
    }
    println!("{total}");
}
