use self::carts::Carts;

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let file = std::fs::read_to_string(filename).expect("can't open file");

    let mut map = Carts::from_str(&file);
    while let None = map.crashes().first() {
        map.step();
    }
    let first_crash = map.crashes()[0];
    println!("Star 1: {},{}", first_crash.x, first_crash.y);

}

mod carts {
    use std::collections::HashMap;
    use self::Direction::*;
    use self::Road::*;

    pub struct Carts {
        map: HashMap<Coords,Road>,
        carts: Vec<Cart>,
        crashes: Vec<Coords>
    }

    impl Carts {
        pub fn from_str(s: &str) -> Carts {
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
            Carts { map, carts, crashes: Vec::new() }
        }
        pub fn step(&mut self) {
            self.carts.sort_by_key(|c| c.location);
            for cart in &mut self.carts {
                cart.location = cart.location.step(cart.direction);
                let road = *self.map
                    .get(&cart.location)
                    .expect("road expected where cart has moved");
                cart.react_to(road);
                // The cart is in its new location now. Handle possible collision.
            }
        }
        pub fn crashes(&self) -> &[Coords] {
            &self.crashes
        }
    }

    #[derive(PartialOrd,Ord,PartialEq,Eq,Clone,Copy,Hash)]
    pub struct Coords {
        pub y: usize,
        pub x: usize
    }

    impl Coords {
        // Attempts to move coords out of grid are not handled or expected.
        fn step(&self, dir: Direction) -> Coords {
            match dir {
                Up => Coords{ x: self.x, y: self.y - 1 },
                Down => Coords{ x: self.x, y: self.y + 1 },
                Left => Coords{ x: self.x - 1, y: self.y },
                Right => Coords{ x: self.x + 1, y: self.y }
            }
        }
    }

    #[derive(PartialEq,Eq,Clone)]
    struct Cart {
        location: Coords,
        direction: Direction,
        next_turn: Turn
    }

    impl Cart {
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
                b'-' | b'^' | b'v' => Some(Horizontal),
                b'|' | b'<' | b'>' => Some(Vertical),
                b'/' => Some(UpRight),
                b'\\' => Some(UpLeft),
                b'+' => Some(Intersection),
                _ => None
            }
        }
    }

}
