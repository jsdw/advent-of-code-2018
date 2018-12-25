use regex::Regex;
use lazy_static::lazy_static;
use std::result;
use std::error::Error;
use std::iter::repeat;
use std::cmp::Reverse;
use self::Weapon::*;

macro_rules! err { ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) } }
type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: String = std::fs::read_to_string(filename)
        .expect("can't open file");

    let armies = parse_armies(&input)?;

    // For star 1, just run the fight and see what happens:
    let (mut army1, mut army2) = armies.clone();
    let remaining_units = fight(&mut army1, &mut army2).expect("fight got stuck");
    println!("Star 1: {}", remaining_units);

    // For star 2, apply boost to first army until it wins:
    let mut remaining_units = 0;
    for boost in 1.. {
        let (mut army1, mut army2) = armies.clone();
        // Apply boost to immune system:
        army1.iter_mut().for_each(|g| g.attack_damage += boost);
        // Ignore fights that get stuck, track remaining otherwise:
        match fight(&mut army1, &mut army2) {
            Some(remaining) => { remaining_units = remaining },
            None => { continue }
        }
        // End when army1 has survived the fight:
        if army1.len() > 0 {
            break;
        }
    }
    println!("Star 2: {}", remaining_units);

    Ok(())
}

// Run fight to completion, returning remaining units. Returns None
// if the fight got stuck and could not complete.
fn fight(army1: &mut Army, army2: &mut Army) -> Option<usize> {
    while army1.len() > 0 && army2.len() > 0 {
        let army1_orders = target_seletion(army1, army2);
        let army2_orders = target_seletion(army2, army1);
        if !attack_phase((army1, army1_orders), (army2, army2_orders)) {
            return None;
        }
    }
    let remaining_units = [&*army1,&*army2]
        .iter()
        .flat_map(|&a| a)
        .map(|g| g.unit_count)
        .sum();
    Some(remaining_units)
}

// Resolve the attacks.
fn attack_phase(a: (&mut Army, Vec<Order>), b: (&mut Army, Vec<Order>)) -> bool {

    let (mut army1, orders1) = a;
    let (mut army2, orders2) = b;

    // associate orders with army identifier so that
    // we can combine and sort them together:
    let g1 = repeat(1).zip(orders1);
    let g2 = repeat(2).zip(orders2);

    // Get group given army and group ID:
    fn get_group<'a>(army: &'a Army, group_id: usize) -> Option<&'a Group> {
        army.iter().find(|g| g.id == group_id)
    };
    fn get_group_mut<'a>(army: &'a mut Army, group_id: usize) -> Option<&'a mut Group> {
        army.iter_mut().find(|g| g.id == group_id)
    };

    // Sort orders by initiative in prep for attack:
    let mut orders: Vec<(usize,Order)> = g1.chain(g2).collect(); 
    orders.sort_by_key(|&(army_ident,(group_id,_))| {
        let army = if army_ident == 1 { &army1 } else { &army2 };
        let group = get_group(army,group_id).unwrap();
        Reverse(group.initiative)
    });

    // Perform each attack:
    let mut successful_attack = false;
    for (army_ident,(group_id,target_id)) in orders {
        let (attackers,defenders) = if army_ident == 1 {
            (&army1,&mut army2)
        } else {
            (&army2, &mut army1)
        };
        let group = match get_group(attackers, group_id) {
            Some(g) => g,
            None => { continue } //group may have died
        };
        let target = match get_group_mut(defenders, target_id) {
            Some(g) => g,
            None => panic!("Target expected with ID {}", target_id)
        };
        if group.attack(target) {
            successful_attack = true;
        }
    }

    // Remove dead groups:
    for army in &mut [&mut army1, &mut army2] {
        for idx in (0..army.len()).rev() {
            if army[idx].is_dead() {
                army.swap_remove(idx);
            }
        }
    }

    // Return false if no attack was successful (ie we are stuck):
    successful_attack
}

type Order = (usize,usize);

// Return a vector of attacker ID to defender ID (if it can attack anything).
// The vector is ordered by which attack should happen first from this side.
fn target_seletion(attackers: &[Group], defenders: &[Group]) -> Vec<Order> {
    let mut choose_order: Vec<&Group> = attackers
        .iter()
        .collect();

    choose_order.sort_by_key(|a| Reverse((a.effective_power(), a.initiative)));

    let mut available_defenders: Vec<&Group> = defenders
        .iter()
        .collect();

    let mut targets_by_id = Vec::new();
    for group in choose_order {
        let target = pick_best_target(group, &available_defenders);
        if let Some(target_id) = target {
            available_defenders
                .iter()
                .position(|d| d.id == target_id)
                .map(|idx| available_defenders.swap_remove(idx));
            targets_by_id.push((group.id, target_id));
        }
    }
    targets_by_id
}

// Pick the target we'd deal most damage to, breaking tie
// by effective power and then initiative:
fn pick_best_target(attacker: &Group, defenders: &[&Group]) -> Option<usize> {
    defenders
        .iter()
        .filter(|d| {
            // Ignore anything we can't deal damage to:
            d.attack_damage_from(attacker) > 0
        })
        .max_by_key(|d| {
            // Pick highest damage/effective power/initiative:
            ( d.attack_damage_from(attacker)
            , d.effective_power()
            , d.initiative )
        })
        .map(|d| d.id)
}

type Army = Vec<Group>;

fn parse_armies(s: &str) -> Result<(Army,Army)> {

    lazy_static!{
        static ref re: Regex = Regex::new(
            r"^(\d+) units each with (\d+) hit points (?:\(([^)]+)\) )?with an attack that does (\d+) (\w+) damage at initiative (\d+)$"
        ).unwrap();
    }

    let mut armies = vec![];
    let mut groups = vec![];
    let mut next_id = 1;

    for line in s.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {

        // New army; save last if present and start new army:
        if line.ends_with(":") {
            if !groups.is_empty() {
                armies.push(groups);
                groups = vec![];
            }
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

        // Add this group to our army with the next available ID:
        let id = next_id;
        next_id += 1;
        groups.push(Group {
            id, unit_count, unit_hp, immune_to, weak_to,
            weapon, attack_damage, initiative
        });
    }
    armies.push(groups);

    if armies.len() != 2 {
        return Err(err!("Expected 2 armies but saw {}", armies.len()));
    }
    Ok((armies.remove(0), armies.remove(0)))
}

#[derive(Debug,Clone)]
struct Group {
    id: usize,
    unit_count: usize,
    unit_hp: usize,
    immune_to: Vec<Weapon>,
    weak_to: Vec<Weapon>,
    weapon: Weapon,
    attack_damage: usize,
    initiative: usize
}

impl Group {
    fn effective_power(&self) -> usize {
        self.unit_count * self.attack_damage
    }
    fn attack_damage_from(&self, attacker: &Group) -> usize {
        if self.immune_to.iter().any(|w| *w == attacker.weapon) {
            0
        } else if self.weak_to.iter().any(|w| *w == attacker.weapon) {
            attacker.effective_power() * 2 
        } else {
            attacker.effective_power()
        }
    }
    fn damage_by(&mut self, d: usize) -> bool {
        let units_killed = d / self.unit_hp;
        if units_killed == 0 {
            return false
        } else if units_killed >= self.unit_count {
            self.unit_count = 0;
        } else {
            self.unit_count -= units_killed;
        }
        true // was the attack successful; did units die?
    }
    fn attack(&self, target: &mut Group) -> bool {
        let damage = target.attack_damage_from(self);
        target.damage_by(damage)
    }
    fn is_dead(&self) -> bool {
        self.unit_count == 0
    }
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