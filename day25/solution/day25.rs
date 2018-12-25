use std::result;
use std::error::Error;
use std::collections::{ HashMap, HashSet };

macro_rules! err { ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) } }
type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let coords: Vec<Coords> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_coords)
        .collect::<Result<_>>()?;

    // What coords is each coord next to:
    let mut adjacent = HashMap::new();
    for c in &coords {
        let mut n = Vec::new();
        for c2 in &coords {
            if distance(c, c2) <= 3 {
                n.push(*c2);
            }
        }
        if n.len() > 0 {
            adjacent.insert(*c, n);
        }
    }

    // Recursively build up constellation counts by
    // following these links:
    let mut visited = HashSet::new();
    let mut count = 0;
    for c in &coords {
        if !visited.contains(c) {
            count += 1;
            visit_neighbours(c, &mut visited, &adjacent);
        }
    }
    println!("Star 1: {}", count);

    Ok(())

}

fn visit_neighbours(coords: &Coords, visited: &mut HashSet<Coords>, adjacent: &HashMap<Coords, Vec<Coords>>) {
    for c in adjacent.get(coords).unwrap_or(&Vec::new()) {
        if !visited.contains(c) {
            visited.insert(*c);
            visit_neighbours(c, visited, adjacent);
        }
    }
}

fn parse_coords(s: &str) -> Result<Coords> {
    let mut cs = [0;4];
    for (idx,c) in s.split(",").enumerate().take(4) {
        let n = c.parse()?;
        cs[idx] = n;
    }
    Ok(cs)
}

type Coords = [i64;4];

fn distance(a: &Coords, b: &Coords) -> i64 {
    a.iter().zip(b).map(|(c1,c2)| (c1-c2).abs()).sum()
}