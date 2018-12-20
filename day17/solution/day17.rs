use std::collections::HashSet;
use self::tracer::Tracer;
use regex::Regex;
use lazy_static::lazy_static;

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let lines: Vec<Line> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_line)
        .collect();

    // Trace out all of the squares filled with water:
    let walls = note_walls(&lines);
    let mut tracer = Tracer::new(walls, 500);
    let bottom = tracer.bottom();
    let top = tracer.top();
    tracer.trace();
    let count_in_bounds = tracer
        .visited()
        .iter()
        .filter(|&&(_,y)| y >= top && y <= bottom)
        .count();
    println!("Star 1: {}", count_in_bounds);

    // Next, work backwards from the bottom removing any water
    // that is not contained (ie left/right/up from bottom water):
    let contained = remove_flowing(tracer);
    let count_contained_in_bounds = contained
        .iter()
        .filter(|&&(_,y)| y >= top && y <= bottom)
        .count();
    println!("Star 2: {}", count_contained_in_bounds);

}

fn remove_flowing(tracer: Tracer) -> HashSet<(usize,usize)> {

    // Define some utility functions to help us remove bits:
    fn remove_from(visited: &mut HashSet<(usize,usize)>, x:usize, mut y:usize) {
        while visited.contains(&(x,y)) {
            visited.remove(&(x,y));
            remove_row(visited, x, y);
            if y > 0 {
                y -= 1;
            } else {
                return;
            }
        }
    }
    fn remove_row(visited: &mut HashSet<(usize,usize)>, x:usize, y:usize) {
        remove_row_direction(visited, x, y, true);
        remove_row_direction(visited, x, y, false);
    }
    fn remove_row_direction(visited: &mut HashSet<(usize,usize)>, x:usize, y:usize, go_right:bool) {
        let inc = |n| if go_right { n + 1 } else { n - 1 };
        let mut next_x = inc(x);
        while visited.contains(&(next_x,y)) {
            visited.remove(&(next_x, y));
            if y > 0 && visited.contains(&(next_x, y-1)) {
                remove_from(visited, next_x, y-1);
            }
            next_x = inc(next_x);
        }
    }

    // Now, find the starting points and remove:
    let bottom = tracer.bottom();
    let mut visited = tracer.into_visited();
    let sinks: Vec<(usize,usize)> = visited.iter()
        .filter(|(_,y)| *y == bottom)
        .cloned()
        .collect();

    for (x,y) in sinks {
        remove_from(&mut visited, x,y);
    }
    visited
}

fn note_walls(lines: &[Line]) -> HashSet<(usize,usize)> {
    let mut walls = HashSet::new();
    for line in lines.iter().cloned() {
        for x in line.x {
            for y in line.y.clone() {
                walls.insert((x,y));
            }
        }
    }
    walls
}

fn parse_line(s: &str) -> Line {
    lazy_static!{
        static ref line_re: Regex = Regex::new(r"(x|y)=(\d+),\s+(x|y)=(\d+)..(\d+)").unwrap();
    }
    let caps = line_re.captures(s).unwrap();
    let first_l = caps.get(1).unwrap().as_str();
    let get_n = |n| caps.get(n).unwrap().as_str().parse().unwrap();

    let fst_r = get_n(2) ..= get_n(2);
    let snd_r = get_n(4) ..= get_n(5);

    let (x,y) = if first_l == "x" {
        (fst_r,snd_r)
    } else {
        (snd_r,fst_r)
    };
    Line { x, y }
}

#[derive(Debug,Clone)]
struct Line {
    x: std::ops::RangeInclusive<usize>,
    y: std::ops::RangeInclusive<usize>
}

mod tracer {
    use std::collections::HashSet;

    pub struct Tracer {
        water_x: usize,
        top: usize,
        bottom: usize,
        sources: HashSet<(usize,usize)>,
        walls: HashSet<(usize,usize)>,
        visited: HashSet<(usize,usize)>
    }
    impl Tracer {
        pub fn new(walls: HashSet<(usize,usize)>, start_x: usize) -> Tracer {
            let top = walls.iter().map(|(_,y)| *y).min().unwrap_or(0);
            let bottom = walls.iter().map(|(_,y)| *y).max().unwrap_or(0);
            Tracer {
                top,
                bottom,
                water_x: start_x,
                walls: walls,
                sources: HashSet::new(),
                visited: HashSet::new()
            }
        }
        pub fn visited(&self) -> &HashSet<(usize,usize)> {
            &self.visited
        }
        pub fn into_visited(self) -> HashSet<(usize,usize)> {
            self.visited
        }
        pub fn top(&self) -> usize {
            self.top
        }
        pub fn bottom(&self) -> usize {
            self.bottom
        }
        #[allow(dead_code)]
        pub fn stringify_region(&self, xs: impl Iterator<Item=usize> + Clone, ys: impl Iterator<Item=usize>) -> String {
            let mut out = String::new();
            for y in ys {
                for x in xs.clone() {
                    let c = if self.visited.contains(&(x,y)) {
                        '~'
                    } else if self.walls.contains(&(x,y)) {
                        '#'
                    } else {
                        '.'
                    };
                    out.push(c);
                }
                out.push('\n');
            }
            out
        }
        pub fn trace(&mut self) {
            self.trace_source(self.water_x, 0);
        }
        fn trace_source(&mut self, x: usize, mut y: usize) {
            if self.sources.contains(&(x,y)) {
                return;
            }
            self.sources.insert((x,y));

            // trace down until we hit a wall:
            while !self.walls.contains(&(x,y+1)) {
                y += 1;
                if y > self.bottom {
                    // we have gone off the bottom; done!
                    return;
                }
                self.visited.insert((x,y));
            }
            // We hit a wall; now trace sideways up from last
            // (x,y) seen to fill in the container, until we
            // overflow it (ie trace over a wall).
            self.fill_container_from(x, y);
        }
        fn fill_container_from(&mut self, x: usize, mut y: usize) {
            loop {
                let right_done = self.fill_direction_from(x, y, true);
                let left_done = self.fill_direction_from(x, y, false);
                if right_done || left_done {
                    return;
                }
                y -= 1;
            }
        }
        fn fill_direction_from(&mut self, x: usize, y: usize, go_right: bool) -> bool {
            let mut next_x = x;
            let mut is_last_source_below = false;
            while !self.walls.contains(&(next_x,y)) {

                let is_source = self.sources.contains(&(next_x,y));
                let is_wall_below = self.walls.contains(&(next_x,y+1));
                let is_visited_below = self.visited.contains(&(next_x,y+1));

                // If there's nothing below us and we just stepped over a source,
                // stop immediately; another source would cover it.
                if !is_visited_below && !is_wall_below && is_last_source_below {
                    return true;
                }

                self.visited.insert((next_x, y));

                // if we are above nothing, we've filled the container
                // and our current location should be a new source to trace down:
                if !is_wall_below && !is_visited_below {
                    self.trace_source(next_x, y);
                    return true;
                }

                next_x = if go_right { next_x + 1 } else { next_x - 1 };
                is_last_source_below = is_source;
            }
            false
        }
    }
}