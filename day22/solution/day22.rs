use crate::cave::Cave;
use crate::solver::Solver;

fn main() {

    let depth = 11739;
    let target = (11,718);

    let cave = Cave::new(depth, target);

    // Work out the risk level for star 1:
    let mut risk_level = 0;
    for y in 0..=target.1 {
        for x in 0..=target.0 {
            risk_level += cave.get((x,y)) as usize;
        }
    }
    println!("Star 1: {}", risk_level);

    // Work out the fastest way to get from 0,0
    // to the target:
    let mut solver = Solver::new(&cave);
    println!("Star 2: {}", solver.solve());

}

// Given a Cave, this module is reposnsible for
// finding the fastest path to the target:
mod solver {
    use crate::cave::Cave;
    use crate::cave::Type::*;
    use self::Tool::*;
    use std::cmp::Reverse;
    use std::collections::HashMap;

    pub struct Solver<'a> {
        cave: &'a Cave,
        current: Vec<State>,
        visited: HashMap<((usize,usize),Tool), usize>,
        best_time: usize
    }

    impl <'a> Solver<'a> {
        pub fn new(cave: &Cave) -> Solver {
            let s = State::starting();
            let d = manhatten_distance(s.position, cave.target());
            let mut visited = HashMap::new();
            visited.insert((s.position,s.tool), s.time_spent);
            Solver {
                cave,
                visited,
                current: vec![s],
                best_time: d * 8, // 1 + 7 to move and change tool every turn.
            }
        }
        pub fn solve(&mut self) -> usize {
            while self.step() { }
            self.best_time
        }
        fn step(&mut self) -> bool {
            // Get next state, bailing if none:
            let first = match self.current.pop() {
                Some(s) => s,
                None => { return false }
            };

            let target = self.cave.target();

            // Keep exploring possible moves if they havent finished and
            // still have a chance to exceed the best time:
            for s in first.possible_moves(self.cave) {

                // // This line speeds the solver up about 10x (~250ms) but no longer
                // // absolutely guarantees a solution, so it's commented out:
                // if s.position.0 > target.0 * 3 || s.position.1 > target.1 * 3 {
                //     continue;
                // }

                // Ignore potential states that could never beat the best:
                let distance = manhatten_distance(s.position, target);
                if distance + s.time_spent >= self.best_time {
                    continue;
                }

                // We're done! have we beaten the best?
                if distance == 0 && s.tool == Torch {
                    self.best_time = self.best_time.min(s.time_spent);
                    continue;
                }

                // If this state is visiting a square we've made it to
                // with a lower score, ignore it:
                let last_seen = self.visited
                    .entry((s.position,s.tool))
                    .or_insert(std::usize::MAX);
                if *last_seen <= s.time_spent {
                    continue;
                }
                *last_seen = s.time_spent;

                // This state has a chance, so add it to the list!
                self.current.push(s);

            }

            // Sort by lowest distance+time-spent-so-far (reversed because
            // we want to order best things to the back to pop them off). We want
            // to get to the end ASAP to constrain which other moves are likely,
            // so we sort in order to make this happen as quickly as possible.
            self.current.sort_unstable_by_key(|s| {
                let distance = manhatten_distance(s.position, target);
                Reverse(distance + s.time_spent)
            });

            true
        }
    }

    #[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
    enum Tool {
        NoTool,
        Torch,
        ClimbingGear
    }

    #[derive(Debug,Clone,Copy)]
    struct State {
        tool: Tool,
        time_spent: usize,
        position: (usize,usize)
    }

    impl State {
        fn starting() -> State {
            State {
                tool: Torch,
                time_spent: 0,
                position: (0,0)
            }
        }
        fn possible_moves(&self, cave: &Cave) -> Vec<State> {
            let mut moves = Vec::with_capacity(5);
            let ty = cave.get(self.position);
            
            // We can change tool to applicable ones:
            let next_possible_tool = match (ty, self.tool) {
                (Rocky, Torch) => ClimbingGear,
                (Rocky, ClimbingGear) => Torch,
                (Wet, NoTool) => ClimbingGear,
                (Wet, ClimbingGear) => NoTool,
                (Narrow, NoTool) => Torch,
                (Narrow, Torch) => NoTool,
                _ => panic!("We shouldn't be using {:?} in {:?} at {:?}", 
                     self.tool, ty, self.position)
            };
            moves.push(State {
                tool: next_possible_tool,
                time_spent: self.time_spent + 7,
                position: self.position
            });

            // We can move to applicable surrounding squares:
            for pos in surrounding_coords(self.position) {
                let nty = cave.get(pos);
                let nstate = State {
                    tool: self.tool,
                    time_spent: self.time_spent + 1,
                    position: pos
                };
                match (nty, self.tool) {
                    (Rocky,Torch)  | (Rocky,ClimbingGear) |
                    (Wet,NoTool)   | (Wet,ClimbingGear)   |
                    (Narrow,Torch) | (Narrow,NoTool)     => moves.push(nstate),
                    _ => { continue }
                }
            } 

            moves
        }
    }

    fn surrounding_coords((x,y): (usize,usize)) -> Vec<(usize,usize)> {
        let mut next = Vec::with_capacity(4);
        next.push((x+1,y));
        next.push((x,y+1));
        if x > 0 { next.push((x-1,y)) }
        if y > 0 { next.push((x,y-1)) }
        next
    }

    fn manhatten_distance((x1,y1): (usize,usize), (x2,y2): (usize,usize)) -> usize {
        let x = if x1 > x2 { x1 - x2 } else { x2 - x1 };
        let y = if y1 > y2 { y1 - y2 } else { y2 - y1 };
        x + y
    }
}

// This allows us to find out what any given piece of cave looks like:
mod cave {
    use std::collections::HashMap;
    use std::cell::RefCell;

    pub struct Cave {
        // Cache known erosions to avoid recalculation:
        erosions: RefCell<HashMap<(usize,usize), usize>>,
        depth: usize,
        target: (usize,usize)
    }

    impl Cave {
        pub fn new(depth: usize, target: (usize,usize)) -> Cave {
            Cave {
                erosions: RefCell::new(HashMap::new()),
                depth,
                target
            }
        }
        pub fn target(&self) -> (usize,usize) {
            self.target
        }
        pub fn get(&self, (x,y):(usize,usize)) -> Type {
            match self.get_erosion(x,y) % 3 {
                0 => Type::Rocky,
                1 => Type::Wet,
                _ => Type::Narrow
            }
        }
        fn get_erosion(&self, x: usize, y: usize) -> usize {
            // return cached value if tis cached:
            if let Some(&e) = self.erosions.borrow().get(&(x,y)) {
                return e;
            }

            // calculate value, cache and return it:
            let geologic_index = if (x,y) == (0,0) {
                0
            } else if (x,y) == self.target {
                0
            } else if y == 0 {
                x * 16807
            } else if x == 0 {
                y * 48271
            } else {
                self.get_erosion(x-1,y) * self.get_erosion(x,y-1)
            };
            let erosion = (geologic_index + self.depth) % 20183;
            self.erosions.borrow_mut().insert((x,y), erosion);
            erosion
        }
    }

    #[derive(Copy,Clone,Debug,PartialEq,Eq)]
    pub enum Type { Rocky, Wet, Narrow }
}