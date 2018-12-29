use regex::Regex;
use lazy_static::lazy_static;
use std::result;
use std::error::Error;

macro_rules! err { ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) } }
type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: Result<Vec<Bot>> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(Bot::from_str)
        .collect();

    let spheres = input?;

    // We've done this already in day23.rs, but I wanted to see how fast an
    // alternative approach for Star2 is. Instead of building a sphere out from
    // (0,0,0) until we touch all of the largest overlapping group, this approach
    // starts witha huge cube that covers enough of the entire area, and breaks it
    // down into smaller cubes, each time checking those that overlap the most, until
    // we have a cube that contains just a single position.
    //
    // This is quicker, taking about 40ms for me rather than ~110ms for the original
    // solution (most of which was spent computing the largest overlapping set), and we
    // could probably make it go faster too.
    let Position{x,y,z} = find_best_overlapping_coords(&spheres);
    println!("Star 2 (alternate approach): {} (at position {},{},{})", x+y+z, x,y,z);
    Ok(())
}

// Find the coordinates closest to origin with the greatest number of overlaps:
fn find_best_overlapping_coords(bots: &[Bot]) -> Position {

    // Roughly work out how big to make our original bounding cube and begin with it:
    let max = bots.iter().map(|s| s.position.from_origin() as usize).max().unwrap();
    let mut searchers = vec![(bots.len(), BoundingCube::new(Position{x:0,y:0,z:0}, max))];

    let mut best_coords = Position {x:0,y:0,z:0};
    let mut best_overlap = 0;
    let mut best_found = false;
    while searchers.len() > 0 {

        let (n,bounds) = searchers.pop().unwrap();

        // Ignore if the best overlapping coords we've found so
        // far are better than these ones already:
        if n < best_overlap {
            continue;
        }

        // Ignore if further away than the best found so far:
        if best_found {
            let this_dist = bounds.distance_from_origin();
            let best_dist = best_coords.from_origin();
            if this_dist >= best_dist {
                continue;
            }
        }

        // If the bounds represent just one position now,
        // it's finished. Is it better than what we have so far?
        if let Some(pos) = bounds.to_position() {
            if !best_found || n > best_overlap {
                best_overlap = n;
                best_coords = pos;
                best_found = true;
            } else if n == best_overlap && pos.from_origin() < best_coords.from_origin() {
                best_coords = pos;
            }
            continue;
        }

        // break the bounding cube down into 8 smaller cubes and add those,
        // caching how many bots they each overlap:
        bounds.split().into_iter()
            .map(|&b| (number_overlapping(&b, &bots),b))
            .for_each(|o| searchers.push(o));

        // Put most desirable next search at the back for the next time:
        searchers.sort_unstable_by_key(|&(n,_)| n);

    }

    best_coords
}

// Count how many bots are overlapping the bounding cube provided.
fn number_overlapping(bounds: &BoundingCube, bots: &[Bot]) -> usize {
    bots.iter().filter(|&bot| bounds.overlaps_bot(bot)).count()
}

// A Bounding cube has a power-of-two size so it can easily divide into
// 8 smaller cubes. The center is the point to the bottom-left of the
// position given rather than a square itself. This is because we are
// drawing a cube around squares. The radius is a power of two so that we
// can easily halve it all the way down to 0. If the radius is 0,  we have
// just a single square left.
#[derive(Debug,Clone,Copy)]
struct BoundingCube {
    center: Position,
    radius: i64
}
impl BoundingCube {
    fn new(center: Position, min_radius: usize) -> BoundingCube {
        BoundingCube {
            center: center,
            radius: min_radius.next_power_of_two() as i64
        }
    }
    fn to_position(&self) -> Option<Position> {
        if self.radius == 0 {
            Some(self.center)
        } else {
            None
        }
    }
    fn closest_to(&self, pos: &Position) -> Position {
        if self.radius == 0 {
            return self.center;
        }
        // -1 because center is bottom-left of square with same coords,
        // but Position given back is a square rather than a point.
        let clamp = |n: i64, c| n.max(c - self.radius).min(c + self.radius - 1);
        let nx = clamp(pos.x, self.center.x);
        let ny = clamp(pos.y, self.center.y);
        let nz = clamp(pos.z, self.center.z);
        Position{ x:nx,y:ny,z:nz }
    }
    fn distance_from(&self, pos: &Position) -> i64 {
        let closest_pos = self.closest_to(pos);
        pos.distance(&closest_pos)
    }
    fn distance_from_origin(&self) -> i64 {
        self.distance_from(&Position{x:0,y:0,z:0})
    }
    fn overlaps_bot(&self, s: &Bot) -> bool {
        let pos = self.closest_to(&s.position);
        s.overlaps_position(&pos)
    }
    fn split(&self) -> [BoundingCube; 8] {
        assert!(self.radius > 0);
        let Position{x,y,z} = self.center;
        let nr = self.radius/2;
        // a radius of 0 corresponds to a bounding cube that covers a
        // single square (should really be a radius of 0.5 to be proper).
        // adjust our movements back to accomodate this case.
        let bb = if nr == 0 { 1 } else { nr };
        let b = |x,y,z| BoundingCube {
            center: Position{x,y,z},
            radius: nr
        };

        [ b(x+nr, y+nr, z+nr)
        , b(x+nr, y+nr, z-bb)
        , b(x+nr, y-bb, z+nr)
        , b(x+nr, y-bb, z-bb)
        , b(x-bb, y+nr, z+nr)
        , b(x-bb, y+nr, z-bb)
        , b(x-bb, y-bb, z+nr)
        , b(x-bb, y-bb, z-bb)
        ]
    }
}

// Bots are centered on some coordinate, and their radius determines how
// far out from that center they can reach in manhatten distance. A bot
// with a radius 0 is 1 block at the position given
#[derive(Debug,Clone,Copy,Eq,PartialEq)]
struct Bot {
    position: Position,
    radius: i64
}
impl Bot {
    fn overlaps_position(&self, pos: &Position) -> bool {
        let dist = self.position.distance(pos);
        dist - self.radius <= 0
    }
    fn from_str(s: &str) -> Result<Bot> {
        lazy_static!{
            static ref re: Regex =
                Regex::new(r"pos=<(-?\d+),(-?\d+),(-?\d+)>, r=(\d+)").unwrap();
        }
        let caps = re.captures(s).ok_or(err!("'{}' not a sphere", s))?;
        let get = |n| caps.get(n).unwrap().as_str().parse().unwrap();
        Ok(Bot {
            position: Position { x: get(1), y: get(2), z: get(3) },
            radius: get(4)
        })
    }
}

// A position in 3 dimensions.
#[derive(Debug,Clone,Copy,Eq,PartialEq,Hash)]
struct Position {
    x: i64,
    y: i64,
    z: i64
}
impl Position {
    fn from_origin(&self) -> i64 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
    fn distance(&self, other: &Position) -> i64 {
        let d1 = (self.x - other.x).abs();
        let d2 = (self.y - other.y).abs();
        let d3 = (self.z - other.z).abs();
        d1 + d2 + d3
    }
}
