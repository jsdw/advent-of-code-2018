use self::interpreter::{ Inputs, Registers, Op, Instruction };
use std::collections::{ HashMap, HashSet };
use regex::Regex;
use lazy_static::lazy_static;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input_string = std::fs::read_to_string(filename)?;
    let input: Input = parse_input(&input_string)?;

    // count how many ops each opcode is seen to represent, and how many
    // opcodes are seen three+ times (for Star 1):
    let mut seen_codes = vec![HashMap::new();16];
    let mut entries_seen_threeplus = 0;
    for Observation{before,instruction,after} in input.observations {
        let ops_seen = &mut seen_codes[instruction.opcode];
        let mut ops_count = 0;

        Op::all().filter(|&op| {
            let ins = instruction.to_instruction(op);
            after == before.apply_instruction(ins)
        }).for_each(|op| {
            *ops_seen.entry(op).or_insert(0) += 1;
            ops_count += 1;
        });

        if ops_count >= 3 {
            entries_seen_threeplus += 1;
        }
    }

    // Collapse seen_codes down into a vec of sets, using counts to
    // filter any ops not seen as often:
    let mut seen_codes: Vec<HashSet<Op>> = seen_codes
        .iter().map(|map| {
            let max = *map.values().max().unwrap();
            map.iter().filter(|t| *t.1 == max).map(|t| *t.0).collect()
        }).collect();

    // While there is still more than one possibility for each
    // opcode, keep trying to cut it down:
    let mut visited = HashSet::new();
    loop {
        let single_item = seen_codes
            .iter()
            .enumerate()
            .filter(|(opcode,_)| !visited.contains(opcode))
            .filter(|(_,s)| s.len() == 1)
            .next();

        match single_item {
            None => break,
            Some((opcode, set)) => {
                let op = *set.iter().next().unwrap();
                visited.insert(opcode);
                seen_codes.iter_mut()
                    .enumerate()
                    .filter(|t| t.0 != opcode)
                    .for_each(|t| { t.1.remove(&op); })
            }
        }
    }

    // We have a vector of sets containing (hopefully) one Op now; flatten
    // to a single vec, blowing up if there is not at least one op per code:
    let seen_codes: Vec<Op> = seen_codes
        .into_iter()
        .map(|mut s| s.drain().next().unwrap())
        .collect();

    // Convert the instructions provided into named instructions
    // and run them on some blank registers:
    let final_registers = input.instructions
        .iter()
        .map(|ins| ins.to_instruction(seen_codes[ins.opcode]))
        .fold(Registers::empty(), |r, ins| r.apply_instruction(ins));

    println!("Star 1: {}", entries_seen_threeplus);
    println!("Star 2: {}", final_registers.get(0));

    Ok(())
}

// Parse our input lines into observations and instructions:
fn parse_input(s: &str) -> Result<Input> {
    lazy_static!{
        static ref before_re: Regex = Regex::new(r"^Before:\s+\[(\d+), (\d+), (\d+), (\d+)\]").unwrap();
        static ref instructions_re: Regex = Regex::new(r"(\d+) (\d+) (\d+) (\d+)").unwrap();
        static ref after_re: Regex = Regex::new(r"^After:\s+\[(\d+), (\d+), (\d+), (\d+)\]").unwrap();
    }

    let mut observations = vec![];
    let mut instructions = vec![];

    fn get(caps: &regex::Captures, n: usize) -> Result<usize> {
        Ok(caps.get(n).expect("cap").as_str().parse()?)
    }
    fn get4<T: From<[usize;4]>>(caps: &regex::Captures) -> Result<T> {
        Ok([get(caps,1)?,get(caps,2)?,get(caps,3)?,get(caps,4)?].into())
    }

    let mut lines = s
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    while let Some(line) = lines.next() {
        if let Some(before_caps) = before_re.captures(line) {
            let line = lines.next().ok_or("line needed for instrs")?;
            let ins_caps = instructions_re.captures(line).ok_or("caps needed for instrs")?;
            let line = lines.next().ok_or("line needed for after")?;
            let after_caps = after_re.captures(line).ok_or("caps needed for after")?;
            observations.push(Observation {
                before: get4(&before_caps)?,
                instruction: get4(&ins_caps)?,
                after: get4(&after_caps)?
            })
        } else if let Some(ins_caps) = instructions_re.captures(line) {
            instructions.push(get4(&ins_caps)?);
        }
    }

    Ok(Input { observations, instructions })
}

#[derive(Clone,Debug)]
pub struct Input {
    pub observations: Vec<Observation>,
    pub instructions: Vec<RawInstruction>
}

#[derive(Copy,Clone,Debug)]
pub struct Observation {
    pub before: Registers,
    pub instruction: RawInstruction,
    pub after: Registers
}

#[derive(Copy,Clone,Debug)]
pub struct RawInstruction {
    pub opcode: usize,
    pub inputs: Inputs
}

impl RawInstruction {
    fn to_instruction(&self, op: Op) -> Instruction {
        Instruction { op, inputs: self.inputs }
    }
}

impl From<[usize;4]> for RawInstruction {
    fn from(input: [usize;4]) -> RawInstruction {
        RawInstruction {
            opcode: input[0],
            inputs: Inputs {
                a: input[1],
                b: input[2],
                c: input[3]
            }
        }
    }
}

// Some of the logic for running instructions against
// registers goes here. This is useful once we finally have
// "real" instructions given the input:
mod interpreter {
    use self::Op::*;

    #[derive(Copy,Clone,Debug,PartialEq,Eq)]
    pub struct Registers([usize;4]);

    impl Registers {
        pub fn empty() -> Registers {
            Registers([0;4])
        }
        pub fn new(inputs: [usize;4]) -> Registers {
            Registers(inputs)
        }
        pub fn apply_instruction(&self, ins: Instruction) -> Registers {
            let mut r = self.0;
            let Inputs { a, b, c } = ins.inputs;
            match ins.op {
                Addr => { r[c] = r[a] + r[b] },
                Addi => { r[c] = r[a] + b },
                Mulr => { r[c] = r[a] * r[b] },
                Muli => { r[c] = r[a] * b },
                Banr => { r[c] = r[a] & r[b] },
                Bani => { r[c] = r[a] & b },
                Borr => { r[c] = r[a] | r[b] },
                Bori => { r[c] = r[a] | b },
                Setr => { r[c] = r[a] },
                Seti => { r[c] = a },
                Gtir => { r[c] = if a > r[b] { 1 } else { 0 } },
                Gtri => { r[c] = if r[a] > b { 1 } else { 0 } },
                Gtrr => { r[c] = if r[a] > r[b] { 1 } else { 0 } },
                Eqir => { r[c] = if a == r[b] { 1 } else { 0 } },
                Eqri => { r[c] = if r[a] == b { 1 } else { 0 } },
                Eqrr => { r[c] = if r[a] == r[b] { 1 } else { 0 } }
            }
            Registers(r)
        }
        pub fn get(&self, n: usize) -> usize {
            self.0[n]
        }
    }

    impl From<[usize;4]> for Registers {
        fn from(inputs: [usize;4]) -> Registers {
            Registers(inputs)
        }
    }

    #[derive(Copy,Clone,Debug)]
    pub struct Instruction {
        pub op: Op,
        pub inputs: Inputs
    }

    #[derive(Copy,Clone,Debug)]
    pub struct Inputs {
        pub a: usize,
        pub b: usize,
        pub c: usize
    }

    #[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
    pub enum Op {
        Addr, Addi,
        Mulr, Muli,
        Banr, Bani,
        Borr, Bori,
        Setr, Seti,
        Gtir, Gtri, Gtrr,
        Eqir, Eqri, Eqrr
    }
    const OPS: [Op; 16] = [
        Addr, Addi,
        Mulr, Muli,
        Banr, Bani,
        Borr, Bori,
        Setr, Seti,
        Gtir, Gtri, Gtrr,
        Eqir, Eqri, Eqrr
    ];
    impl Op {
        pub fn all() -> impl Iterator<Item=Op> {
            OPS.iter().cloned()
        }
    }
}