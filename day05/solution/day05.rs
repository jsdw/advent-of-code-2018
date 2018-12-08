fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: Vec<u8> = std::fs::read(filename).expect("can't open file");

    // Star 1: do a reaction and see what the length is:
    println!("Star 1: {}", reaction_length(&input));

    // Star 2: try removing each letter and then react. What's
    // the shortest length that we can achieve:
    let shortest = (b'a' ..= b'z')
        .map(|c| {
            reaction_length(input.iter().filter(|&&b| b != c && b != c - 32))
        })
        .min()
        .unwrap();

    println!("Star 2: {:?}", shortest);

}

// Iterate through the chars, reacting any that would react with the current
// last char. Return the final length.
fn reaction_length<'a>(input: impl IntoIterator<Item=&'a u8>) -> usize {
    let mut cs = vec![];
    for &c in input.into_iter() {
        match cs.last() {
            Some(&c2) if (c as i16 - c2 as i16).abs() == 32 => { cs.pop(); },
            _ => { cs.push(c); }
        };
    }
    cs.len()
}