fn main() {
    (1..10).zip(1..5).for_each(|(i, j)| println!("{} {}", i, j));
}
