use self::plants::Plants;
use std::collections::HashMap;

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let file = std::fs::read_to_string(filename)
        .expect("can't open file");

    let (initial, patterns) = extract_data_from_file(&file);

    // Star 1: just do 20 steps and count plant numbers:
    let mut plants = Plants::from_iter(initial.clone());
    for _ in 0..20 {
        plants.step(&patterns);
    }
    let sum: i64 = plants.positions().sum();
    println!("Star 1: {}", sum);

    // Star 2: Given some observation, I notice that after a certain
    // step, the values start being identical after every step and just
    // the offset is different, so find the values/offset and fast-forward!
    let mut plants = Plants::from_iter(initial.clone());
    let mut last_values = plants.values();
    let mut last_offset = plants.offset();
    let mut next_offset = plants.offset();
    let mut last_step = 0;
    for step in 1.. {
        plants.step(&patterns);
        let next_values = plants.values();
        let next_step = step;
        next_offset = plants.offset();
        if next_values == last_values {
            break;
        } else {
            last_values = next_values;
            last_offset = next_offset;
            last_step = next_step;
        }
    }
    let offset_diff = (50_000_000_000 - last_step - 1) * (next_offset - last_offset);
    let sum: i64 = plants.positions().map(|p| p + offset_diff).sum();
    println!("Star 2: {}", sum);

}

mod plants {
    use std::collections::VecDeque;

    pub struct Plants {
        offset: i64,
        values: VecDeque<bool>
    }

    impl Plants {
        pub fn from_iter(it: impl IntoIterator<Item=bool>) -> Plants {
            Plants {
                offset: 0,
                values: it.into_iter().collect()
            }
        }
        pub fn step(&mut self, pats: &[[bool;5]]) {
            self.add_padding();
            let old = &self.values;
            let mut new = self.values.clone();
            for idx in 2..new.len()-2 {
                let curr = [old[idx-2],old[idx-1],old[idx],old[idx+1],old[idx+2]];
                if pats.iter().any(|p| *p == curr) {
                    new[idx] = true;
                } else {
                    new[idx] = false;
                }
            }
            self.values = new;
            self.remove_padding();
        }
        pub fn positions(&mut self) -> impl Iterator<Item=i64> + '_ {
            self.values
                .iter()
                .zip(self.offset..)
                .filter(|(b,_)| **b)
                .map(|(_,i)| i)
        }
        pub fn values(&self) -> Vec<bool> {
            self.values.iter().cloned().collect()
        }
        pub fn offset(&self) -> i64 {
            self.offset
        }
        fn add_padding(&mut self) {
            let first_n = self.values
                .iter()
                .position(|b| *b)
                .unwrap_or(self.values.len()-1);
            let last_n = self.values
                .iter().rev()
                .position(|b| *b)
                .unwrap_or(0);

            if first_n < 5 {
                (0..5-first_n).for_each(|_| {
                    self.values.push_front(false);
                    self.offset -= 1;
                })
            }
            if last_n < 5 {
                (0..5-last_n).for_each(|_| {
                    self.values.push_back(false);
                })
            }
        }
        fn remove_padding(&mut self) {
            while let Some(false) = self.values.front() {
                self.values.pop_front();
                self.offset += 1;
            }
            while let Some(false) = self.values.back() {
                self.values.pop_back();
            }
        }
    }

}

fn extract_data_from_file(file: &str) -> (Vec<bool>, Vec<[bool;5]>) {
    let mut lines = file.lines().filter(|l| !l.is_empty());

    let initial = lines
        .next()
        .expect("initial input")
        .bytes()
        .filter_map(to_bool)
        .collect();

    let yes_patterns = lines
        .filter_map(to_yes_pattern)
        .collect();

    (initial, yes_patterns)
}

fn to_yes_pattern(line: &str) -> Option<[bool;5]> {
    let mut pieces = line.split(" => ");
    let bools = pieces.next().unwrap().bytes().filter_map(to_bool);
    let res = pieces.next().unwrap().bytes().filter_map(to_bool).next().unwrap();

    if !res {
        return None;
    }

    let mut pat = [false;5];
    for (b, p) in bools.zip(pat.iter_mut()) {
        *p = b
    }
    Some(pat)
}

fn to_bool(byte: u8) -> Option<bool> {
    match byte {
        b'#' => Some(true),
        b'.' => Some(false),
        _ => None
    }
}