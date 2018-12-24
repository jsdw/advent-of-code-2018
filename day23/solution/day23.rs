use regex::Regex;
use lazy_static::lazy_static;
use std::result;
use std::error::Error;
use std::collections::{ HashMap, HashSet };

macro_rules! err { ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) } }
type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: Result<Vec<Sphere>> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(Sphere::from_str)
        .collect();

    let spheres = input?;

    // For star 1, find the largest of these spheres and count how many other
    // spheres are in range, including itself:
    let largest_sphere = spheres
        .iter()
        .enumerate()
        .max_by_key(|(_,s)| s.radius)
        .map(|(i,_)| spheres[i])
        .unwrap();
    let largest_overlaps = spheres
        .iter()
        .filter(|s| largest_sphere.in_range(&s.position))
        .count();
    println!("Star 1: {}", largest_overlaps);

    // For star 2, find the largest set of total overlapping spheres. Ignore
    // spheres that contain other spheres in the list. Then grow a sphere out
    // from (0,0,0) until it overlaps with all of them.
    let spheres = largest_overlapping_set(&spheres)?;
    let origin = Position { x:0, y:0, z:0 };
    let furthest = spheres.iter()
        .map(|s| s.position.distance(&origin) - s.radius)
        .max().unwrap();

    let mut overlapping_radii = (furthest..).filter(|&radius| {
        let s = Sphere { position: origin, radius };
        let num_overlapping = number_overlapping(&s, &spheres);
        num_overlapping == spheres.len()
    });
    println!("Star 2: {}", overlapping_radii.next().unwrap());

    Ok(())
}

fn number_overlapping(sphere: &Sphere, spheres: &[Sphere]) -> usize {
    spheres.iter().filter(|s| s.overlaps_with(sphere)).count()
}

fn largest_overlapping_set(spheres: &[Sphere]) -> Result<Vec<Sphere>> {

    // Which spheres overlap with eachother? As an optimisation, we
    // only compare later spheres in the list for overlap; The earliest
    // in a list of overlaps will always have all of the potential 
    // overlaps in the group even if later ones do not.
    let mut overlaps = HashMap::new();
    for (idx1,s1) in spheres.iter().enumerate() {
        let mut os = HashSet::new();
        for (idx2,s2) in spheres[idx1+1 .. ].iter().enumerate() {
            if s1.overlaps_with(s2) {
                os.insert(idx2);
            }
        }
        if os.len() > 0 {
            overlaps.insert(idx1, os);
        }
    }

    // For each sphere, find the set of total overlaps (ie
    // the fully connected sub-graph) from here.
    let mut full_overlaps: Vec<Vec<usize>> = Vec::new();
    for (&idx1, os) in &overlaps {
        let mut full: HashSet<usize> = HashSet::new();
        for &idx2 in os {
            let overlaps_with_everything = full
                .iter()
                .all(|&idx| spheres[idx].overlaps_with(&spheres[idx2]));
            if overlaps_with_everything {
                full.insert(idx2);
            }
        }
        full.insert(idx1);
        let v: Vec<_> = full.into_iter().collect();
        full_overlaps.push(v);
    }

    full_overlaps
        .into_iter()
        .max_by_key(|v| v.len())
        .map(|v| v.into_iter().map(|idx| spheres[idx]).collect())
        .ok_or(err!("no spheres"))

}


#[derive(Debug,Clone,Copy,Eq,PartialEq)]
struct Sphere {
    position: Position,
    radius: i64
}

impl Sphere {
    fn from_str(s: &str) -> Result<Sphere> {
        lazy_static!{
            static ref re: Regex = 
                Regex::new(r"pos=<(-?\d+),(-?\d+),(-?\d+)>, r=(\d+)").unwrap();
        }
        let caps = re.captures(s).ok_or(err!("'{}' not a sphere", s))?;
        let get = |n| caps.get(n).unwrap().as_str().parse().unwrap();
        Ok(Sphere {
            position: Position { x: get(1), y: get(2), z: get(3) },
            radius: get(4)
        })
    }
    fn in_range(&self, other: &Position) -> bool {
        self.position.distance(other) <= self.radius
    }
    fn overlaps_with(&self, other: &Sphere) -> bool {
        let dist = self.position.distance(&other.position);
        dist <= self.radius + other.radius
    }
}

#[derive(Debug,Clone,Copy,Eq,PartialEq,Hash)]
struct Position {
    x: i64,
    y: i64,
    z: i64
}

impl Position {
    fn distance(&self, other: &Position) -> i64 {
        let d1 = (self.x - other.x).abs();
        let d2 = (self.y - other.y).abs();
        let d3 = (self.z - other.z).abs();
        d1 + d2 + d3
    }
}