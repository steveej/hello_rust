/// Quick implementation of a friend's homework excercise
fn main() {
    let original: [[usize; 4]; 4] = [
        [61, 84, 126, 297],
        [33, 80, 600, 392],
        [2, 5, 7, 1],
        [12, 63, 71, 16],
    ];

    let interval_bottom = 50;
    let interval_top = 150;

    let mut result: Vec<(usize, usize, usize)> = Vec::new();
    let mut sum = 0;
    let mut count = 0;

    for i in 0..4 {
        for j in 0..4 {
            let value = original[i][j];
            if value >= interval_bottom && value <= interval_top {
                result.push((i, j, value));
                sum += value;
                count += 1;
            }
        }
    }

    println!("{:?}\nsum: {}\ncount: {}", result, sum, count);
}