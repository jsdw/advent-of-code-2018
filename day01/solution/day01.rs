use std::env;
use std::fs;
use std::collections::HashSet;

fn main() {

    let filename = env::args().nth(1).expect("need puzzle input");

    let numbers = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(|b| b.parse().expect("valid number"))
        .collect::<Vec<i64>>();


    // star 1:
    println!("Star 1: {}", numbers.iter().fold(0, |sum, &n| sum + n));

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
