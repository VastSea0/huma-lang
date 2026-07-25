fn main() {
    let mut values: Vec<u64> = Vec::with_capacity(50_000);
    for i in 0..50_000 {
        values.push(i % 997);
    }
    let total: u64 = values.iter().sum();
    println!("{total}");
}
