use std::collections::HashSet;
use self::simulation::Simulation;
use regex::Regex;
use lazy_static::lazy_static;

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let lines: Vec<Line> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_line)
        .collect();

    let walls = draw_walls(&lines);
    let mut sim = Simulation::new(walls, 500);
    let top = sim.top();
    let bottom = sim.bottom();
    let mut last_water_count = 0;
    let mut round = 0;
    loop {
        sim.step();
        let c = sim.water().len();
        round += 1;
        if round % 100 == 0 {
            println!("Round {}, water: {}", round, c);
            print_region(&sim, 430..530, 160..260);
        }

        if c == last_water_count {
            break;
        } else {
            last_water_count = c;
        }
    }
    let count_in_bounds = sim
        .water()
        .iter()
        .filter(|&&(_,y)| y >= top && y <= bottom )
        .count();
    println!("Star 1: {}", count_in_bounds);

}

fn draw_walls(lines: &[Line]) -> HashSet<(usize,usize)> {
    let mut walls = HashSet::new();
    for mut line in lines.iter().cloned() {
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

fn print_region(sim: &Simulation, xs: impl Iterator<Item=usize> + Clone, ys: impl Iterator<Item=usize>) -> Result<(),std::io::Error> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let water = sim.water();
    let walls = sim.walls();
    for y in ys {
        for x in xs.clone() {
            let c = if water.contains(&(x,y)) {
                "~"
            } else if walls.contains(&(x,y)) {
                "#"
            } else {
                "."
            };
            write!(&mut handle, "{}", c)?;
        }
        writeln!(&mut handle)?;
    }
    Ok(())
}

#[derive(Debug,Clone)]
struct Line {
    x: std::ops::RangeInclusive<usize>,
    y: std::ops::RangeInclusive<usize>
}

// A simple water simulation:
//
mod simulation {
    use std::collections::HashSet;

    pub struct Simulation {
        water_x: usize,
        top: usize,
        bottom: usize,
        walls: HashSet<(usize,usize)>,
        water: HashSet<(usize,usize)>
    }

    impl Simulation {
        pub fn new(walls: HashSet<(usize,usize)>, water_x: usize) -> Simulation {
            let top = walls.iter().map(|(x,_)| *x).min().unwrap_or(0);
            let bottom = walls.iter().map(|(_,y)| *y).max().unwrap_or(0);
            let water = HashSet::new();
            Simulation{ water_x, walls, top, bottom, water }
        }
        pub fn step(&mut self) {
            let mut water_coords: Vec<(usize,usize)> = self.water.iter().cloned().collect();
            // move water from the bottom up:
            water_coords.sort_unstable_by(|(_,y1),(_,y2)| y1.cmp(y2).reverse());
            for (x,y) in water_coords {
                self.step_water(x,y)
            }
            self.water.insert((self.water_x,0));
        }
        pub fn walls(&self) -> &HashSet<(usize,usize)> {
            &self.walls
        }
        pub fn water(&self) -> &HashSet<(usize,usize)> {
            &self.water
        }
        pub fn top(&self) -> usize {
            self.top
        }
        pub fn bottom(&self) -> usize {
            self.bottom
        }
        fn step_water(&mut self, x:usize,y:usize) {
            // water is at bottom of grid so it falls off:
            if y >= self.bottom {
                self.water.remove(&(x,y));
                return;
            }
            // wall beneath water, so can't move:
            else if self.walls.contains(&(x,y+1)) {
                return;
            }
            // water beneath water so try to fall beside:
            else if self.water.contains(&(x,y+1)) {
                if let Some((nx,ny)) = self.find_space_beside_water(x,y+1) {
                    self.water.remove(&(x,y));
                    self.water.insert((nx,ny));
                }
            }
            // nothing beneath water so move it down one:
            else {
                self.water.remove(&(x,y));
                self.water.insert((x,y+1));
            }
        }
        fn find_space_beside_water(&self, x:usize, y:usize) -> (Option<(usize,usize)> {
            // find the next available space to the left and right, if any:
            let mut more_left  = true;
            let mut more_right = true;
            let mut x_left  = x;
            let mut x_right = x;
            let mut found_left = None;
            let mut found_right = None;
            while more_left || more_right {
                if more_left {
                    if self.walls.contains(&(x_left, y)) {
                        more_left = false;
                    } else if self.water.contains(&(x_left, y)) {
                        if x_left == 0 { more_left = false }
                        else { x_left -= 1 }
                    } else {
                        more_left = false;
                        found_left = Some((x_left, y));
                    }
                }
                if more_right {
                    if self.walls.contains(&(x_right, y)) {
                        more_right = false;
                    } else if self.water.contains(&(x_right, y)) {
                        x_right += 1;
                    } else {
                        more_right = false;
                        found_right = Some((x_right, y));
                    }
                }
            }

            // pick one of these spaces, prioritising one with something underneath it.
            // we add a bit of randomness to ensure all routes are eventually travelled.
            match (found_left, found_right) {
                (None, Some(_)) => found_right,
                (Some(_), None) => found_left,
                (None, None) => None,
                (Some((lx,ly)), Some((rx,ry))) => {
                    if rand::random() {
                        found_left
                    } else {
                        found_right
                    }
                }
            }
        }
    }
}