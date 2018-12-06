use std::collections::{ HashMap };

fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: Vec<Coord> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_coords)
        .collect();

    // Find the bounding box containing all inputs:
    let bounds = find_bounds(&input);

    // Find out how many coords in this bounding box are
    // closest to each input coord:
    let mut closest = HashMap::new();
    for x in bounds.left ..= bounds.right {
        for y in bounds.top ..= bounds.bottom {
            if let Some(v) = find_closest(&Coord{x,y}, &input) {
                *closest.entry(v).or_insert(0) += 1;
            }
        }
    }

    // Find out which input is closest to each coord
    // just outside the box, so we know which inputs
    // will have an infinite area and can ignore them
    let mut remove_input = |c| {
        if let Some(v) = c {
            closest.remove(&v);
        }
    };
    for x in bounds.left ..= bounds.right {
        let y = bounds.top - 1;
        remove_input(find_closest(&Coord{x,y}, &input));
        let y = bounds.bottom + 1;
        remove_input(find_closest(&Coord{x,y}, &input));
    }
    for y in bounds.top ..= bounds.bottom {
        let x = bounds.left - 1;
        remove_input(find_closest(&Coord{x,y}, &input));
        let x = bounds.right + 1;
        remove_input(find_closest(&Coord{x,y}, &input));
    }

    // Finally, which of the remaining coords in our closest
    // list is closer to the most places in our bounding box,
    // and what number of places is that?
    let biggest_area = closest.values().max().unwrap();
    println!("Star 1: {}", biggest_area);

    // For part two, work out the total distance of all inputs
    // to each square in our bounding box, and count the number
    // of squares that then have a total distance less than 10000:
    let mut region_cells = 0;
    for x in bounds.left ..= bounds.right {
        for y in bounds.top ..= bounds.bottom {
            let t: i32 = input
                .iter()
                .map(|c| c.distance_to(&Coord{x,y}))
                .sum();
            if t < 10000 {
                region_cells += 1
            }
        }
    }
    println!("Star 2: {}", region_cells);

}

fn parse_coords(s: &str) -> Coord {
    let mut parts = s.split(", ");
    Coord {
        x: parts.next().unwrap().parse().unwrap(),
        y: parts.next().unwrap().parse().unwrap()
    }
}

fn find_closest(c: &Coord, points: &[Coord]) -> Option<Coord> {
    let mut distance = std::i32::MAX;
    let mut closest = None;
    for point in points {
        let d = point.distance_to(c);
        if d == distance {
            closest = None;
        } else if d < distance {
            distance = d;
            closest = Some(*point);
        }
    }
    closest
}

fn find_bounds(coords: &[Coord]) -> Bounds {
    let mut bounds = Bounds {
        top: std::i32::MAX,
        left: std::i32::MAX,
        bottom: std::i32::MIN,
        right: std::i32::MIN
    };
    for c in coords {
        if c.x < bounds.left {
            bounds.left = c.x
        } else if c.x > bounds.right {
            bounds.right = c.x
        }
        if c.y < bounds.top {
            bounds.top = c.y
        } else if c.y > bounds.bottom {
            bounds.bottom = c.y
        }
    }
    bounds
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
struct Coord { x: i32, y: i32 }

impl Coord {
    fn distance_to(&self, other: &Coord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug)]
struct Bounds { top: i32, left: i32, bottom: i32, right: i32 }