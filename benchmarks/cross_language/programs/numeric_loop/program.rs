fn main() {
    let mut state: u64 = 1;
    let mut total: u64 = 0;
    for _ in 1..=200_000 {
        state = (state * 1_664_525 + 1_013_904_223) % 4_294_967_296;
        total = (total + state) % 9_007_199_254_740_881;
    }
    println!("{total}");
}
