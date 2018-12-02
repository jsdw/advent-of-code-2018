fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let file: String = std::fs::read_to_string(filename).expect("can't open file");
    let lines: Vec<&str> = file.lines().collect();

    // star 1:
    let mut two_of = 0;
    let mut three_of = 0;
    for &line in &lines {
        let mut counts = [0;26];
        for byte in line.bytes() {
            counts[byte as usize - 97] += 1;
        }

        let mut seen_two = false;
        let mut seen_three = false;
        for &c in counts.into_iter() {
            if !seen_two && c == 2 {
                two_of += 1;
                seen_two = true;
            }
            if !seen_three && c == 3 {
                three_of += 1;
                seen_three = true;
            }
        }
    }
    println!("Star 1: {}", two_of * three_of);

    // Star 2:
    let mut matching_chars = String::new();
    'outer: for (l_idx, line) in lines.iter().enumerate() {
        let this_chars: Vec<char> = line.chars().collect();
        for other_line in lines[l_idx..].iter() {
            let mut diff = 0;
            let mut diff_idx = 0;
            for (c_idx, c) in other_line.chars().enumerate() {
                if this_chars[c_idx] != c {
                    diff += 1;
                    diff_idx = c_idx;
                }
            }
            if diff == 1 {
                matching_chars = other_line
                    .chars()
                    .enumerate()
                    .filter(|&(idx,_)| idx != diff_idx)
                    .map(|(_,c)| c)
                    .collect();
                break 'outer;
            }
        }
    }
    println!("Star 2: {}", matching_chars);

}
