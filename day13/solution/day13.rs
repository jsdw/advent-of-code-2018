use self::carts::State;

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let file = std::fs::read_to_string(filename).expect("can't open file");

    // First, find the first crash location:
    {
        let mut map = State::from_str(&file);
        while map.crashes().len() == 0 {
            map.step();
        }
        let first_crash = map.crashes()[0];
        println!("Star 1: {},{}", first_crash.x, first_crash.y);
    }

    // Next, find the last cart standing:
    {
        let mut map = State::from_str(&file);
        while map.carts().len() > 1 {
            map.step();
        }
        let last_loc = map.carts()[0].loc();
        println!("Star 2: {},{}", last_loc.x, last_loc.y);
    }

}

// All of our logic to do with carts and such lives in this module:
mod carts {
    use std::collections::HashMap;
    use self::Direction::*;
    use self::Road::*;

    // ### This is our current state: ###
    pub struct State {
        map: HashMap<Coords,Road>,
        carts: Vec<Cart>,
        crashes: Vec<Coords>
    }
    impl State {
        pub fn from_str(s: &str) -> State {
            let mut map = HashMap::new();
            let mut carts = Vec::new();
            for (y, line) in s.lines().enumerate() {
                for (x, byte) in line.bytes().enumerate() {
                    if let Some(road) = Road::from_byte(byte) {
                        map.insert(Coords{x,y}, road);
                    }
                    if let Some(dir) = Direction::from_byte(byte) {
                        carts.push(Cart {
                            next_turn: Turn::Left,
                            location: Coords{x,y},
                            direction: dir
                        });
                    }
                }
            }
            State { map, carts, crashes: Vec::new() }
        }
        pub fn step(&mut self) {

            // In order from top to bottom, move each cart if we can:
            self.carts.sort_by_key(|c| c.location);
            let mut visited = HashMap::new();
            for cart in &mut self.carts {
                let n = visited.entry(cart.location).or_insert(0);
                // seen another cart at this loc; mark crash, don't move:
                if *n > 0 {
                    *n += 1;
                }
                // move the cart and then increment location counter:
                else {
                    cart.location.step(cart.direction);
                    *visited.entry(cart.location).or_insert(0) += 1;
                    let road = *self.map.get(&cart.location).unwrap();
                    cart.react_to(road);
                }
            }

            // add any locations used more than once as crashes:
            for (&loc,&count) in &visited {
                if count > 1 {
                    self.crashes.push(loc);
                }
            }

            // keep remaining carts and add to crashed list:
            let carts = std::mem::replace(&mut self.carts, Vec::new());
            let mut next_carts = vec![];
            for cart in carts {
                if *visited.get(&cart.location).unwrap_or(&0) > 1 {
                    self.crashes.push(cart.location);
                } else {
                    next_carts.push(cart);
                }
            }
            self.crashes.dedup();
            self.carts = next_carts;

        }
        pub fn crashes(&self) -> &[Coords] {
            &self.crashes
        }
        pub fn carts(&self) -> &[Cart] {
            &self.carts
        }
    }

    // ### X,Y location (y is first as we sort by it first) ###
    #[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Clone,Copy,Hash)]
    pub struct Coords {
        pub y: usize,
        pub x: usize
    }
    impl Coords {
        // Attempts to move coords out of grid are not handled or expected.
        fn step(&mut self, dir: Direction) {
            match dir {
                Up    => { self.y -= 1 },
                Down  => { self.y += 1 },
                Left  => { self.x -= 1 },
                Right => { self.x += 1 }
            }
        }
    }

    // ### A Single cart ###
    #[derive(PartialEq,Eq,Clone)]
    pub struct Cart {
        location: Coords,
        direction: Direction,
        next_turn: Turn
    }
    impl Cart {
        pub fn loc(&self) -> Coords {
            self.location
        }
        fn react_to(&mut self, road: Road) {
            match road {
                UpRight => {
                    self.direction = match self.direction {
                        Up => Right,
                        Right => Up,
                        Left => Down,
                        Down => Left
                    };
                },
                UpLeft => {
                    self.direction = match self.direction {
                        Up => Left,
                        Right => Down,
                        Left => Up,
                        Down => Right
                    };
                },
                Intersection => {
                    self.direction = self.direction.turn(self.next_turn);
                    self.next_turn = self.next_turn.next_turn();
                },
                _ => {
                    /* don't need to change anything */
                }
            }
        }
    }

    // ### What direction to turn at an intesection ###
    #[derive(PartialEq,Eq,Clone,Copy)]
    enum Turn {
        Left,
        Straight,
        Right
    }
    impl Turn {
        fn next_turn(&self) -> Turn {
            match self {
                Turn::Left => Turn::Straight,
                Turn::Straight => Turn::Right,
                Turn::Right => Turn::Left
            }
        }
    }

    // ### What direction are we facing ###
    #[derive(PartialEq,Eq,Clone,Copy)]
    enum Direction {
        Up,
        Down,
        Right,
        Left
    }
    impl Direction {
        fn from_byte(b: u8) -> Option<Direction> {
            match b {
                b'^' => Some(Up),
                b'>' => Some(Right),
                b'v' => Some(Down),
                b'<' => Some(Left),
                _ => None
            }
        }
        fn turn_left(&self) -> Direction {
            match self {
                Up => Left,
                Left => Down,
                Down => Right,
                Right => Up
            }
        }
        fn turn_right(&self) -> Direction {
            match self {
                Up => Right,
                Right => Down,
                Down => Left,
                Left => Up,
            }
        }
        fn turn(&self, turn: Turn) -> Direction {
            match turn {
                Turn::Left => self.turn_left(),
                Turn::Right => self.turn_right(),
                Turn::Straight => *self
            }
        }
    }

    // ### A piece of road ###
    #[derive(PartialEq,Eq,Clone,Copy)]
    enum Road {
        Horizontal,
        Vertical,
        UpRight,
        UpLeft,
        Intersection
    }
    impl Road {
        fn from_byte(b: u8) -> Option<Road> {
            match b {
                b'-' | b'<' | b'>' => Some(Horizontal),
                b'|' | b'v' | b'^' => Some(Vertical),
                b'/' => Some(UpRight),
                b'\\' => Some(UpLeft),
                b'+' => Some(Intersection),
                _ => None
            }
        }
    }

}
