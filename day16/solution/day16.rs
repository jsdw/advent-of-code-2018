use self::interpreter::{ Inputs, Registers, Op, Instruction };
use regex::Regex;
use lazy_static::lazy_static;
use std::error::Error;

// macro_rules! err { ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) } }
type Result<T> = std::result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input_string = std::fs::read_to_string(filename)?;
    let input: Input = parse_input(&input_string)?;

    let like_3plus = input.observations
        .iter()
        .map(|&Observation{before,instruction,after}| {
            Op::all().filter(|&op| {
                let ins = Instruction{ op, inputs: instruction.inputs };
                after == before.apply_instruction(ins)
            }).count()
        })
        .filter(|&n| n >= 3)
        .count();
    println!("Star 1: {}", like_3plus);

    Ok(())
}

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

// We handle interpreter specific stuff here:
mod interpreter {
    use self::Op::*;

    #[derive(Copy,Clone,Debug,PartialEq,Eq)]
    pub struct Registers([usize;4]);

    impl Registers {
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
                Seti => { r[a] = a },
                Gtir => { r[c] = if a > r[b] { 1 } else { 0 } },
                Gtri => { r[c] = if r[a] > b { 1 } else { 0 } },
                Gtrr => { r[c] = if r[a] > r[b] { 1 } else { 0 } },
                Eqir => { r[c] = if a == r[b] { 1 } else { 0 } },
                Eqri => { r[c] = if r[a] == b { 1 } else { 0 } },
                Eqrr => { r[c] = if r[a] == r[b] { 1 } else { 0 } }
            }
            Registers(r)
        }
        pub fn into_array(self) -> [usize;4] {
            self.0
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

    #[derive(Copy,Clone,Debug)]
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