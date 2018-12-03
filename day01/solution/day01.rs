use std::collections::HashSet;

fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");

    let numbers = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(|b| b.parse().expect("valid number"))
        .collect::<Vec<i64>>();

    // star 1:
    let offset: i64 = numbers.iter().sum();
    println!("Star 1: {}", offset);

    // star 2:
    let mut freq = 0;
    let mut seen = HashSet::new();
    seen.insert(0);
    for n in numbers.into_iter().cycle() {
        freq += n;
        if !seen.insert(freq) {
            break;
        }
    }
    println!("Star 2: {}", freq);

}
