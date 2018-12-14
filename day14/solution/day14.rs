
type Recipe = u128;

fn main() {
    let input = 110201;

    let next_10 = scores(3, 7)
        .skip(input)
        .take(10)
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("");
    println!("Digits: {}", next_10);

}

// Create infinite list of scores
fn scores(first: u128, second: u128) -> impl Iterator<Item=u128> {
    // NOT IMPLEMENTED YET
    (0..=9).cycle()
}

// Turns 12345 into vector [1,2,3,4,5]
fn digits(value: u128) -> Vec<u128> {
    let mut v: Vec<_> = (0..)
        .map(move |n| value / 10u128.pow(n))
        .take_while(|&n| n > 0)
        .map(|n| n % 10)
        .collect();

    v.reverse();
    v
}
