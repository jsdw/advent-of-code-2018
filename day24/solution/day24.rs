use regex::Regex;
use lazy_static::lazy_static;
use std::result;
use std::error::Error;
use std::collections::{ HashMap, HashSet };
use self::Weapon::*;

macro_rules! err { ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) } }
type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: String = std::fs::read_to_string(filename)
        .expect("can't open file");

    let (army1,army2) = parse_armies(&input)?;

    println!("{:?}\n\n{:?}", army1, army2);

    Ok(())
}


fn parse_armies(s: &str) -> Result<(Army,Army)> {

    lazy_static!{
        static ref re: Regex = Regex::new(
            r"^(\d+) units each with (\d+) hit points (?:\(([^)]+)\) )?with an attack that does (\d+) (\w+) damage at initiative (\d+)$"
        ).unwrap();
    }

    let mut armies = vec![];
    let mut army = Army::empty();

    for line in s.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {

        // New army; save last if present and start new army:
        if line.ends_with(":") {
            if !army.is_empty() {
                armies.push(army);
                army = Army::empty();
            }
            army.name = line.trim_end_matches(":").to_owned();
            continue;
        }

        // Units in army; parse and add to army:
        let caps = re.captures(line).ok_or(err!("Line not a valid unit: '{}'", line))?;
        let get = |n| caps.get(n).unwrap().as_str();
        let unit_count = get(1).parse()?;
        let unit_hp = get(2).parse()?;
        let attack_damage = get(4).parse()?;
        let weapon = Weapon::from_str(get(5))?;
        let initiative = get(6).parse()?;

        // Parse optional immunities/weaknesses if provided:
        let mut immune_to = Vec::new();
        let mut weak_to = Vec::new();
        if let Some(cs) = caps.get(3) {
            for bit in cs.as_str().split("; ") {
                let immune_prefix = "immune to ";
                let weak_prefix = "weak to ";
                if bit.starts_with(immune_prefix) {
                    for w in bit.trim_start_matches(immune_prefix).split(", ") {
                        immune_to.push(Weapon::from_str(w)?);
                    }
                }
                if bit.starts_with(weak_prefix) {
                    for w in bit.trim_start_matches(weak_prefix).split(", ") {
                        weak_to.push(Weapon::from_str(w)?);
                    }
                }
            }
        }

        // Add this group to our army:
        army.groups.push(Group {
            unit_count, unit_hp, immune_to, weak_to,
            weapon, attack_damage, initiative
        });
    }
    armies.push(army);

    if armies.len() != 2 {
        return Err(err!("Expected 2 armies but saw {}", armies.len()));
    }
    Ok((armies.pop().unwrap(), armies.pop().unwrap()))
}

#[derive(Debug,Clone)]
struct Army {
    name: String,
    groups: Vec<Group>
}

impl Army {
    fn empty() -> Army {
        Army {
            name: String::new(),
            groups: vec![]
        }
    }
    fn is_empty(&self) -> bool {
        self.name.is_empty() && self.groups.is_empty()
    }
}

#[derive(Debug,Clone)]
struct Group {
    unit_count: usize,
    unit_hp: usize,
    immune_to: Vec<Weapon>,
    weak_to: Vec<Weapon>,
    weapon: Weapon,
    attack_damage: usize,
    initiative: usize
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum Weapon {
    Cold,
    Slashing,
    Radiation,
    Bludgeoning,
    Fire
}

impl Weapon {
    fn from_str(s: &str) -> Result<Weapon> {
        match s {
            "cold" => Ok(Cold),
            "slashing" => Ok(Slashing),
            "radiation" => Ok(Radiation),
            "bludgeoning" => Ok(Bludgeoning),
            "fire" => Ok(Fire),
            _ => Err(err!("Weapon variant not found for '{}'", s))
        }
    }
}