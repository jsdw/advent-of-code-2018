
fn main() {

    let input = 110201;
    let next_10 = scores(3, 7)
        .skip(input)
        .take(10)
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("");
    println!("Star 1: {}", next_10);

    let input_digits = vec![1,1,0,2,0,1];
    let count = windowed(input_digits.len(), scores(3,7))
        .take_while(|ns| ns != &input_digits)
        .count();
    println!("Star 2: {}", count);

}

// Create iterator over scores:
fn scores(first: u8, second: u8) -> impl Iterator<Item=u8> {

    let mut elf1_pos = 0;
    let mut elf2_pos = 1;
    let mut scores = vec![first, second];

    let it = (0..).flat_map(move |_| {

        let elf1_score = scores[elf1_pos];
        let elf2_score = scores[elf2_pos];
        let combined = elf1_score + elf2_score;

        let ns = if combined > 9 {
            vec![combined / 10, combined % 10]
        } else {
            vec![combined]
        };

        for &s in &ns {
            scores.push(s)
        }

        elf1_pos = (elf1_pos + 1 + elf1_score as usize) % scores.len();
        elf2_pos = (elf2_pos + 1 + elf2_score as usize) % scores.len();

        ns

    });

    vec![first,second].into_iter().chain(it)
}

// Create windowed iterator that shows n at a time:
fn windowed<T: Clone>(n: usize, mut it: impl Iterator<Item=T>) -> impl Iterator<Item=Vec<T>> {

    let mut window = vec![];
    let mut do_initial = true;
    let producer = move |_| {
        if do_initial {
            do_initial = false;
            for _ in 0..n {
                if let Some(t) = it.next() {
                    window.push(t)
                } else {
                    return None;
                }
            }
        } else {
            if let Some(t) = it.next() {
                window.remove(0);
                window.push(t);
            } else {
                return None;
            }
        }
        Some(window.clone())
    };

    (0..)
        .map(producer)
        .take_while(|n| n.is_some())
        .map(|n| n.unwrap())
        .fuse()

}