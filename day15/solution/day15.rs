use self::battle::{State,Opts};

fn main() {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: String = std::fs::read_to_string(filename).expect("can't open file");

    // Star 1: How many rounds would things last with damage of 3:
    let (round, total_health) = run_part1(&input);
    println!("Star 1: {} ({} rounds, {} health)",  total_health * round, round, total_health);

    // Star 2: Keep upping elf damage until they win without a loss. What's the outcome?
    let (round, total_health, damage) = run_part2(&input);
    println!("Star 2: {} ({} rounds, {} health, damage {})",  total_health * round, round, total_health, damage);
}

// Star 1:
fn run_part1(s: &str) -> (i32, i32) {
    let opts = Opts::new().elf_damage(3);
    let mut state = State::from_str(&s, opts);
    run(&mut state)
}

// Star 2:
fn run_part2(s: &str) -> (i32, i32, i32) {
    let mut damage = 4;
    loop {
        let opts = Opts::new().elf_damage(damage).stop_on_elf_death(true);
        let mut state = State::from_str(&s, opts);
        let (round, total_health) = run(&mut state);
        if !state.did_an_elf_die() {
            return (round, total_health, damage);
        } else {
            damage += 1;
        }
    }
}

// Run a given state to completion and return the outcome:
fn run(state: &mut State) -> (i32,i32) {
    let mut round = 0;
    while state.round() {
        round += 1;
    }
    let total_health: i32 = state.units().map(|u| u.health).sum();
    (round, total_health)
}

// ##################################################################
// # This module contains the things we need to simulate the battle #
// ##################################################################
mod battle {
    use std::collections::{ HashMap, HashSet };
    use std::fmt::{self,Display};
    use self::UnitType::*;

    const STARTING_HEALTH: i32 = 200;

    // Opts used to initialise State:
    //
    #[derive(Copy,Clone,Debug)]
    pub struct Opts {
        elf_damage: i32,
        stop_on_elf_death: bool
    }
    impl Opts {
        pub fn new() -> Opts {
            Opts {
                elf_damage: 3,
                stop_on_elf_death: false
            }
        }
        pub fn elf_damage(mut self, d: i32) -> Opts {
            self.elf_damage = d;
            self
        }
        pub fn stop_on_elf_death(mut self, b: bool) -> Opts {
            self.stop_on_elf_death = b;
            self
        }
    }

    // Our game state:
    //
    pub struct State {
        units: HashMap<Coords, Unit>,
        walls: HashSet<Coords>,
        stop_on_elf_death: bool,
        starting_elves: i32,
        elves_remaining: i32,
        goblins_remaining: i32
    }
    impl State {
        pub fn from_str(s: &str, opts: Opts) -> State {
            let mut units = HashMap::new();
            let mut walls = HashSet::new();
            let mut starting_elves = 0;
            let mut starting_goblins = 0;
            for (y, line) in s.trim().lines().enumerate() {
                for (x, byte) in line.trim().bytes().enumerate() {
                    let p = Coords{ x:x as i32, y:y as i32 };
                    match byte {
                        b'#' => { walls.insert(p); },
                        b'E' => {
                            units.insert(p, Unit{
                                ty:Elf,
                                damage:opts.elf_damage,
                                health:STARTING_HEALTH
                            });
                            starting_elves += 1;
                        },
                        b'G' => {
                            units.insert(p, Unit{
                                ty:Goblin,
                                damage:3,
                                health:STARTING_HEALTH
                            });
                            starting_goblins += 1;
                        },
                         _  => { /* ignore other bits */ }
                    }
                }
            }
            State {
                units,
                walls,
                starting_elves,
                elves_remaining: starting_elves,
                goblins_remaining: starting_goblins,
                stop_on_elf_death: opts.stop_on_elf_death
            }
        }
        pub fn round(&mut self) -> bool {

            // Sort units by reading order so that we know how to progress:
            let mut unit_coords: Vec<Coords> = self.units.keys().cloned().collect();
            unit_coords.sort();

            // Track dead units so we don't try to use those coords:
            let mut dead_units = HashSet::new();

            // Bail out if there's going to be nothing to do:
            if self.is_finished() {
                return false;
            }

            let mut finished_early = false;
            for mut coords in unit_coords {

                // If the unit at these coords was recently killed,
                // don't try to get it (you may succeed, but get a
                // unit that's since moved into the free space!):
                if dead_units.contains(&coords) {
                    continue;
                }

                // Nothing left to do but we have at least one more unit to move,
                // so bail out and make a note that the round failed to finish:
                if finished_early {
                    return false;
                }

                // Get unit (if it's not been killed!)
                let unit = *self.units.get(&coords).expect("unit expected");

                // Who is this unit fighting?
                let enemy_ty = if unit.ty == Elf { Goblin } else { Elf };

                // Move the unit if there is a path to move along:
                if let Some(new_coords) = self.step_to_nearest_unit(coords, enemy_ty) {
                    self.units.remove(&coords);
                    self.units.insert(new_coords, unit);
                    coords = new_coords;
                }

                // Attack if we're near enough to an enemy:
                if let Some(enemy_coords) = self.adjacent_unit_to_attack(coords, enemy_ty) {
                    let enemy = self.units.get_mut(&enemy_coords).unwrap();
                    enemy.health -= unit.damage;
                    if enemy.health <= 0 {
                        match enemy.ty {
                            Elf => { self.elves_remaining -= 1; },
                            Goblin => { self.goblins_remaining -= 1; }
                        }
                        self.units.remove(&enemy_coords);
                        dead_units.insert(enemy_coords);
                        finished_early = self.is_finished();
                    }
                }
            }
            true
        }
        pub fn units(&self) -> impl Iterator<Item=Unit> + '_ {
            self.units.values().cloned()
        }
        pub fn is_finished(&self) -> bool {
            (self.stop_on_elf_death && self.did_an_elf_die()) || self.is_battle_over()
        }
        pub fn is_battle_over(&self) -> bool {
            self.elves_remaining == 0 || self.goblins_remaining == 0
        }
        pub fn did_an_elf_die(&self) -> bool {
            self.elves_remaining < self.starting_elves
        }
        fn adjacent_units(&self, coords: Coords, ty: UnitType) -> impl Iterator<Item=(Coords,Unit)> + '_ {
            coords.adjacent()
                .into_iter()
                .filter_map(move |c| {
                    let unit = *self.units.get(&c)?;
                    if unit.ty != ty { return None };
                    Some((c, unit))
                })
        }
        fn adjacent_unit_to_attack(&self, coords: Coords, ty: UnitType) -> Option<Coords> {
            self.adjacent_units(coords, ty)
                // when attacking, find unit with lowest health first, reading order if tie:
                .min_by_key(|(c,unit)| (unit.health, *c))
                .map(|(c,_)| c)
        }
        fn next_available_coords<'a>(&'a self, coords: Coords) -> impl Iterator<Item=Coords> + 'a {
            coords.adjacent()
                .into_iter()
                .filter(move |c| !self.walls.contains(c))
                .filter(move |c| !self.units.contains_key(c))
        }
        fn step_to_nearest_unit(&self, start_coords: Coords, ty: UnitType) -> Option<Coords> {

            // Find the nearest enemy, recording the distance from start as we go:
            let mut visited = HashMap::new();
            visited.insert(start_coords, 0);
            let next_to_enemy = {
                let mut current = vec![start_coords];
                let mut maybe_next_to_enemy: Option<Coords> = None;
                while !current.is_empty() {

                    // Find all squares next to an enemy. pick the one that's
                    // best for reading order:
                    maybe_next_to_enemy = current.iter()
                        .filter(|&c| self.adjacent_units(*c,ty).next().is_some())
                        .min()
                        .map(|&c| c);
                    if maybe_next_to_enemy.is_some() {
                        break;
                    }

                    // step our coords one square along and record:
                    let mut next = vec![];
                    for coord in current {
                        let current_dist = *visited.get(&coord).unwrap();
                        for next_coord in self.next_available_coords(coord) {
                            if !visited.contains_key(&next_coord) {
                                visited.insert(next_coord, current_dist+1);
                                next.push(next_coord);
                            }
                        }
                    }
                    current = next;
                }
                maybe_next_to_enemy?
            };

            // Trace back from enemy square to beginning, forming a path that's
            // the shortest possible and is the best reader order:
            let next_coords = {
                let mut path = vec![next_to_enemy];
                loop {
                    let coord = *path.last().unwrap();
                    if coord == start_coords {
                        break;
                    }
                    let next_coord = coord.adjacent()
                        .into_iter()
                        .filter(|c| visited.contains_key(c))
                        // min distance. min coord if tied distance:
                        .min_by_key(|c| (visited.get(c).unwrap(), *c))
                        .expect("next expected");
                    path.push(next_coord);
                }
                path.pop(); // remove the current location.
                path.last().map(|&c| c) // use the one just after.
            };
            next_coords
        }
    }

    // pretty print our state to help with debug:
    impl Display for State {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut height = 0;
            let mut width = 0;

            for coord in &self.walls {
                height = height.max(coord.y + 1);
                width = width.max(coord.x + 1);
            }

            for y in 0..height {
                let mut units = vec![];
                for x in 0..width {
                    let pos = Coords{x,y};
                    if self.walls.contains(&pos) {
                        write!(f, "#")?;
                    } else if let Some(unit) = self.units.get(&pos) {
                        if unit.ty == Elf {
                            write!(f, "E")?;
                        } else {
                            write!(f, "G")?;
                        }
                        units.push(unit);
                    } else {
                        write!(f, ".")?;
                    }
                }
                for unit in units {
                    if unit.ty == Elf {
                        write!(f, " E({})", unit.health)?;
                    } else {
                        write!(f, " G({})", unit.health)?;
                    }
                }
                write!(f, "\n")?;
            }
            Ok(())
        }
    }

    #[derive(Debug,Clone,Copy,Eq,PartialEq,Ord,PartialOrd,Hash)]
    pub struct Coords {
        y: i32, // y first for reading order
        x: i32
    }
    impl Coords {
        pub fn adjacent(&self) -> Vec<Coords> {
            let Coords { x, y } = *self;
            vec![Coords{x:x,y:y-1}, Coords{x:x-1,y:y}, Coords{x:x+1,y:y}, Coords{x:x,y:y+1}]
        }
    }

    #[derive(Copy,Clone,Debug)]
    pub struct Unit {
        pub ty: UnitType,
        pub health: i32,
        pub damage: i32
    }

    #[derive(Copy,Clone,Eq,PartialEq,Debug)]
    pub enum UnitType {
        Elf,
        Goblin
    }

}

// ###############################################
// # A bunch of tests to help ensure correctness #
// ###############################################
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1() {
        assert_eq!(run_part1(r"
            #######
            #.G...#
            #...EG#
            #.#.#G#
            #..G#E#
            #.....#
            #######
        "), (47, 590));

        assert_eq!(run_part1(r"
            #######
            #G..#E#
            #E#E.E#
            #G.##.#
            #...#E#
            #...E.#
            #######
        "), (37, 982));

        assert_eq!(run_part1(r"
            #######
            #E..EG#
            #.#G.E#
            #E.##E#
            #G..#.#
            #..E#.#
            #######
        "), (46, 859));

        assert_eq!(run_part1(r"
            #######
            #E.G#.#
            #.#G..#
            #G.#.G#
            #G..#.#
            #...E.#
            #######
        "), (35, 793));

        assert_eq!(run_part1(r"
            #######
            #.E...#
            #.#..G#
            #.###.#
            #E#G#G#
            #...#G#
            #######
        "), (54, 536));

        assert_eq!(run_part1(r"
            #########
            #G......#
            #.E.#...#
            #..##..G#
            #...##..#
            #...#...#
            #.G...G.#
            #.....G.#
            #########
        "), (20, 937));
    }

    #[test]
    fn part1_extra() {
        assert_eq!(run_part1(r"
            ####
            ##E#
            #GG#
            ####
        "), (67, 200));

        assert_eq!(run_part1(r"
            #####
            #GG##
            #.###
            #..E#
            #.#G#
            #.E##
            #####
        "), (71, 197));

        assert_eq!(run_part1(r"
            ################
            #.......G......#
            #G.............#
            #..............#
            #....###########
            #....###########
            #.......EG.....#
            ################
        "), (38, 486));

        assert_eq!(run_part1(r"
            ################################
            #########...####################
            #########...###########.########
            #########G..##########....######
            ##########..###########...######
            #########G...##########...######
            #########..G.###########..######
            ########...#.##########..#######
            #######G#..###E######....#######
            #######G.....#.######....#######
            ######...G......##E......#######
            ####...##.#..G..G.........######
            ###..........G#####.......####.#
            ####........G#######...........#
            ####..G.....#########......#...#
            ###.........#########........###
            ##.....G.G..#########......#####
            #...G.......#########.........##
            #.G.........#########.E.##...###
            ##.....G.....#######....G#.E...#
            ##............#####...E.......##
            #.G...........E.......#E...##.##
            #....G........###########.....##
            #......##...#.##################
            #.#.........E..##.##############
            #.#.......G.......##############
            #.###........E....##############
            #.####.....###....##############
            #.#####......E..################
            #######..........###############
            #########..####.################
            ################################
        "), (149,2326));

        assert_eq!(run_part1(r"
            ################################
            #################.....##########
            #################..#.###########
            #################.........######
            ##################......########
            #################G.GG###########
            ###############...#..###########
            ###############......G..########
            ############..G.........########
            ##########.G.....G......########
            ##########......#.........#..###
            ##########...................###
            #########G..G.#####....E.G.E..##
            ######..G....#######...........#
            #######.....#########.........##
            #######..#..#########.....#.####
            ##########..#########..G.##..###
            ###########G#########...E...E.##
            #########.G.#########..........#
            #########GG..#######.......##.E#
            ######.G......#####...##########
            #...##..G..............#########
            #...#...........###..E.#########
            #.G.............###...##########
            #................###############
            ##.........E.....###############
            ###.#..............#############
            ###..G........E.....############
            ###......E..........############
            ###......#....#E#...############
            ###....####.#...##.#############
            ################################
        "), (68,2803));
    }

    #[test]
    fn part2() {
        assert_eq!(run_part2(r"
            #######
            #.G...#
            #...EG#
            #.#.#G#
            #..G#E#
            #.....#
            #######
        "), (29, 172, 15));

        assert_eq!(run_part2(r"
            #######
            #E..EG#
            #.#G.E#
            #E.##E#
            #G..#.#
            #..E#.#
            #######
        "), (33, 948, 4));

        assert_eq!(run_part2(r"
            #######
            #E.G#.#
            #.#G..#
            #G.#.G#
            #G..#.#
            #...E.#
            #######
        "), (37, 94, 15));

        assert_eq!(run_part2(r"
            #######
            #.E...#
            #.#..G#
            #.###.#
            #E#G#G#
            #...#G#
            #######
        "), (39, 166, 12));

        assert_eq!(run_part2(r"
            #########
            #G......#
            #.E.#...#
            #..##..G#
            #...##..#
            #...#...#
            #.G...G.#
            #.....G.#
            #########
        "), (30, 38, 34));
    }

    #[test]
    fn part2_extra() {
        assert_eq!(run_part2(r"
            ################################
            ###########################..###
            ##########################...###
            #########################..#####
            ####...##################.######
            #####..################...#.####
            #..G...G#########.####G.....####
            #.......########.....G.......###
            #.....G....###G....#....E.....##
            ####...##......##.............##
            ####G...#.G...###.G...........##
            ####G.......................####
            ####.........G#####.........####
            ####...GG#...#######.......#####
            ###.........#########G....######
            ###.G.......#########G...#######
            ###.G.......#########......#####
            ####.....G..#########....E..####
            #####.......#########..E....####
            ######...##G.#######........####
            ######.#.#.G..#####.....##..####
            ########....E...........##..####
            ########....E#######........####
            ########......######E....##..E.#
            ########......#####.....#......#
            ########.....######............#
            ##################...#.E...E...#
            ##################.............#
            ###################.......E#####
            ####################....#...####
            ####################.###########
            ################################
        "),(46,1457,20));

    }

}
