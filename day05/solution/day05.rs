fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: Vec<u8> = std::fs::read(filename).expect("can't open file");

    // Star 1: do a reaction and see what the length is:
    println!("Star 1: {}", reaction_length(input.clone()));

    // Star 2: try removing each letter and then react. What's
    // the shortest length that we can achieve:
    let shortest = (b'a' ..= b'z')
        .map(|c| {
            let new_input = input
                .iter()
                .cloned()
                .filter(|&b| b != c && b != c - 32)
                .collect();

            reaction_length(new_input)
        })
        .min()
        .unwrap();

    println!("Star 2: {:?}", shortest);

}

// just remove elements who are different case but same letter
// until there's nothing left to remove. Treat as ascii and this is easy,
// tho Vec not the best structure to use:
fn reaction_length(mut input: Vec<u8>) -> usize {
    let mut i = 0;
    while i < input.len() - 1 {
        let a = input[i];
        let b = input[i+1];

        if (a as i16 - b as i16).abs() == 32 {
            input.drain(i..=i+1).for_each(|_| ());
            if i > 0 { i -= 1 }
        } else {
            i += 1;
        }
    }
    input.len()
}