use self::grid::Grid;
use self::simulation::{Simulation,Item};
use regex::Regex;
use lazy_static::lazy_static;

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let lines: Vec<Line> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_line)
        .collect();

    let grid = draw_lines(&lines);
    let mut sim = Simulation::new(grid, 500);
    let mut last_water_count = 0;
    loop {
        sim.step();
        let c = sim
            .grid()
            .values()
            .filter(|(_,v)| **v == Item::Water)
            .count();
        if c == last_water_count {
            break;
        } else {
            last_water_count = c;
        }
    }
    println!("Star 1: {}", last_water_count);

}

fn draw_lines(lines: &[Line]) -> Grid<Item> {
    let (w,h) = lines.iter().fold((0,0), |(w,h), line| {
        (w.max(*line.x.end()), h.max(*line.y.end()))
    });

    let mut grid = Grid::new(w+2,h+1,Item::Empty);
    for mut line in lines.iter().cloned() {
        for x in &mut line.x {
            for y in &mut line.y {
                grid[(x,y)] = Item::Wall;
            }
        }
    }
    grid
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

// A simple water simulation:
//
mod simulation {
    use crate::grid::Grid;

    pub struct Simulation {
        water_x: usize,
        grid: Grid<Item>
    }

    impl Simulation {
        pub fn new(grid: Grid<Item>, water_x: usize) -> Simulation {
            Simulation{ water_x, grid }
        }
        pub fn step(&mut self) {
            // move each water square if possible:
            for y in self.grid.height()-1 .. 0 {
                for x in self.grid.width()-1 .. 0 {
                    self.step_square(x,y);
                }
            }
            // add some water if possible:
            match self.grid[(self.water_x,0)] {
                Item::Empty => {
                    self.grid[(self.water_x,0)] = Item::Water;
                },
                Item::Water => {
                    if let Some((nx,ny)) = self.find_space_beside_water(self.water_x, 0) {
                        self.grid[(nx,ny)] = Item::Water;
                    }
                },
                Item::Wall => {
                    panic!("Water hole is a wall!");
                }
            }
        }
        pub fn grid(&self) -> &Grid<Item> {
            &self.grid
        }
        fn step_square(&mut self, x:usize,y:usize) {
            // square isn't water so do nothing:
            if self.grid[(x,y)] != Item::Water {
                return;
            }
            // square is at bottom of grid so it falls off:
            if y >= self.grid.height() - 1 {
                self.grid[(x,y)] = Item::Empty;
                return;
            }
            match self.grid[(x,y+1)] {
                // square below is wall so can't move:
                Item::Wall => {
                    return;
                },
                // square below is empty so move water there:
                Item::Empty => {
                    self.grid[(x,y)] = Item::Empty;
                    self.grid[(x,y+1)] = Item::Water;
                },
                // square below is water so see if we can fall beside it:
                Item::Water => {
                    if let Some((nx,ny)) = self.find_space_beside_water(x,y+1) {
                        self.grid[(x,y)] = Item::Empty;
                        self.grid[(nx,ny)] = Item::Water;
                    }
                }
            }
        }
        fn find_space_beside_water(&self, x:usize, y:usize) -> Option<(usize,usize)> {
            // *** Prioritise spaces with water or wall beneath them over empty spaces ***
            let max_x = self.grid.width() - 1;
            let mut no_more_left  = false;
            let mut no_more_right = false;
            let mut x_left  = x;
            let mut x_right = x;
            while !no_more_left || !no_more_right {
                if !no_more_left {
                    match self.grid[(x_left, y)] {
                        Item::Empty => { return Some((x_left, y)) },
                        Item::Wall => { no_more_left = true; },
                        Item::Water => { if x_left == 0 { no_more_left = true } else { x_left -= 1 } }
                    }
                }
                if !no_more_right {
                    match self.grid[(x_right,y)] {
                        Item::Empty => { return Some((x_right, y)) },
                        Item::Wall => { no_more_right = true; },
                        Item::Water => { if x_right == max_x { no_more_right = true } else { x_right += 1 } }
                    }
                }
            }
            None
        }
    }

    #[derive(Debug,Copy,Clone,PartialEq,Eq)]
    pub enum Item {
        Wall,
        Water,
        Empty
    }

}

// A simple 2D grid:
//
mod grid {

    pub struct Grid<T> {
        width: usize,
        height: usize,
        values: Vec<T>
    }

    impl <T> Grid<T> {
        pub fn new(width: usize, height: usize, value: T) -> Grid<T>
        where T: Clone {
            Grid {
                width,
                height,
                values: vec![value; width * height]
            }
        }
        pub fn width(&self) -> usize {
            self.width
        }
        pub fn height(&self) -> usize {
            self.height
        }
        pub fn values(&self) -> impl Iterator<Item=((usize,usize),&T)> {
            (0..self.height).flat_map(move |y| {
                (0..self.width).map(move |x| ((x,y),&self.values[y * self.width + x]))
            }).rev()
        }
        // pub fn values_mut<'a>(&'a mut self) -> impl Iterator<Item=((usize,usize),&'a mut T)> + 'a {
        //     (0..self.height).flat_map(move |y| {
        //         (0..self.width).map(move |x| ((x,y),&mut self.values[y * self.width + x]))
        //     }).rev()
        // }

    }

    impl <T> std::ops::Index<(usize,usize)> for Grid<T> {
        type Output = T;
        fn index(&self, (x,y): (usize,usize)) -> &T {
            &self.values[y * self.width + x]
        }
    }
    impl <T> std::ops::IndexMut<(usize,usize)> for Grid<T> {
        fn index_mut(&mut self, (x,y): (usize,usize)) -> &mut T {
            &mut self.values[y * self.width + x]
        }
    }

}