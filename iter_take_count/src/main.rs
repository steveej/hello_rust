fn main() {
    assert_eq!(1, (1..10).step_by(1).filter(|i| i < &5).take(1).count());
}
