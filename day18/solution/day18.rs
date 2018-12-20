use std::collections::HashMap;

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: String = std::fs::read_to_string(filename).expect("can't open file");

    // 10 generations:
    let mut acres = parse_acres(&input);
    for _ in 0..10 {
        acres = step_acres(&acres);
    }
    println!("Star 1: {}", calculate_score(&acres));

    // find repetition to work out state after 1billion:
    let mut acres = parse_acres(&input);
    let mut seen = HashMap::new();
    let (fst,nxt) = (0..).filter_map(|idx| {
        let v = acres_to_vec(&acres);
        seen.get(&v).and_then(|&fst| {
            Some((fst,idx))
        }).or_else(|| {
            seen.insert(v, idx);
            acres = step_acres(&acres);
            None
        })
    }).next().unwrap();

    // Repetition found; do the final rounds we need and job done:
    let rounds_to_do = (1_000_000_000 - nxt) % (nxt - fst);
    for _ in 0..rounds_to_do {
        acres = step_acres(&acres);
    }
    println!("Star 2: {}", calculate_score(&acres));

}

fn calculate_score(acres: &HashMap<Coord,Item>) -> usize {
    let (w,l) = acres.values().fold((0,0), |(w,l), item| {
        match item {
            Item::Lumberyard => (w,l+1),
            Item::Trees => (w+1,l),
            Item::Open => (w,l)
        }
    });
    w * l
}

fn parse_acres(s: &str) -> HashMap<Coord,Item> {
    let mut h = HashMap::new();
    for (y, line) in s.lines().enumerate() {
        for (x, byte) in line.bytes().enumerate() {
            let item = match byte {
                b'.' => Item::Open,
                b'|' => Item::Trees,
                b'#' => Item::Lumberyard,
                _ => panic!("Character not supported: '{}'", byte as char)
            };
            h.insert(Coord{x,y}, item);
        }
    }
    h
}

fn print_acres(acres: &HashMap<Coord,Item>) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for y in 0..50 {
        for x in 0..50 {
            write!(&mut handle, "{}", match acres.get(&Coord{x,y}) {
                Some(Item::Open) => ".",
                Some(Item::Lumberyard) => "#",
                Some(Item::Trees) => "|",
                None => "X"
            }).unwrap();
        }
        write!(&mut handle, "\n");
    }
}

fn step_acres(acres: &HashMap<Coord,Item>) -> HashMap<Coord,Item> {
    let mut h = HashMap::new();
    for x in 0..50 {
        for y in 0..50 {
            let c = Coord{x,y};
            h.insert(c, step_item(c, acres));
        }
    }
    h
}

fn step_item(c: Coord, acres: &HashMap<Coord,Item>) -> Item {
    let item = acres.get(&c).expect("item expected");
    let surrounds = count_surrounding(&c, acres);
    match item {
        Item::Open => {
            if surrounds.trees >= 3 {
                Item::Trees
            } else {
                Item::Open
            }
        },
        Item::Trees => {
            if surrounds.lumbaryard >= 3 {
                Item::Lumberyard
            } else {
                Item::Trees
            }
        },
        Item::Lumberyard => {
            if surrounds.lumbaryard >= 1 && surrounds.trees >= 1 {
                Item::Lumberyard
            } else {
                Item::Open
            }
        }
    }
}

fn count_surrounding(c: &Coord, acres: &HashMap<Coord,Item>) -> Surrounding {
    let mut open = 0;
    let mut trees = 0;
    let mut lumbaryard = 0;
    for s in c.surrounding() {
        match acres.get(&s) {
            Some(Item::Open) => { open += 1 },
            Some(Item::Trees) => { trees += 1 },
            Some(Item::Lumberyard) => { lumbaryard += 1 },
            None => { /* ignore */ }
        }
    }
    Surrounding { open, trees, lumbaryard }
}

fn acres_to_vec(acres: &HashMap<Coord,Item>) -> Vec<Item> {
    let mut n = Vec::with_capacity(50 * 50);
    for x in 0..50 {
        for y in 0..50 {
            n.push(*acres.get(&Coord{x,y}).unwrap());
        }
    }
    n
}

#[derive(Eq,PartialEq,Copy,Clone,Debug)]
struct Surrounding {
    open: usize,
    trees: usize,
    lumbaryard: usize
}

#[derive(Hash,Eq,PartialEq,Copy,Clone,Debug)]
enum Item {
    Open,
    Trees,
    Lumberyard
}

#[derive(Hash,Eq,PartialEq,Copy,Clone,Debug)]
struct Coord {
    x: usize,
    y: usize
}

impl Coord {
    pub fn surrounding(&self) -> Vec<Coord> {
        let Coord{x,y} = *self;
        let mut cs = vec![];
        cs.push(Coord{x:x+1,y:y});
        cs.push(Coord{x:x+1,y:y+1});
        cs.push(Coord{x:x,y:y+1});
        if x > 0 {
            cs.push(Coord{x:x-1,y:y});
            cs.push(Coord{x:x-1,y:y+1});
        }
        if y > 0 {
            cs.push(Coord{x:x,y:y-1});
            cs.push(Coord{x:x+1,y:y-1});
        }
        if x > 0 && y > 0 {
            cs.push(Coord{x:x-1,y:y-1});
        }
        cs
    }
}