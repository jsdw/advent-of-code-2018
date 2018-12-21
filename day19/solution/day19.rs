use self::interpreter::{ Inputs, Instruction, Interpreter, Op };
use std::error::Error;
use std::result;

macro_rules! err { ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) } }
type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: String = std::fs::read_to_string(filename).expect("can't open file");
    let Input { pointer_register, instructions } = parse_input(&input)?;

    let mut interpreter = Interpreter::new(instructions.clone(), pointer_register);
    while interpreter.step() { }
    println!("Star 1: {}", interpreter.registers()[0]);

    /*
    // The instructions provided amount to the below code.
    // My "num" is 10551403, which has 4 factors according
    // to Wolfram alpha. Summing them gives the answer to
    // Star 2...
    var total = 0;
    var num = 10551403;
    var a = 1;
    while (a <= num) {
        var b = 1;
        while (b <= num) {
            if (a * b == num) {
                total = a + total // summing the factors!
            }
            b++;
        }
        a++;
    }
    */

    Ok(())
}

fn parse_input(s: &str) -> Result<Input> {
    let mut pointer_register = 0;
    let mut instructions = vec![];
    for line in s.lines() {
        if line.starts_with("#ip") {
            pointer_register = line.chars().skip(4).collect::<String>().parse()?;
        } else {
            instructions.push(parse_instruction(line)?);
        }
    }
    Ok(Input { pointer_register, instructions })
}

fn parse_instruction(s: &str) -> Result<Instruction> {
    let bits: Vec<&str> = s.split(char::is_whitespace).collect();

    if bits.len() != 4 {
        return Err(err!("Expecting 4 pieces, got: '{:?}'", bits));
    }

    let op = match bits[0] {
        "addr" => Op::Addr,
        "addi" => Op::Addi,
        "mulr" => Op::Mulr,
        "muli" => Op::Muli,
        "banr" => Op::Banr,
        "bani" => Op::Bani,
        "borr" => Op::Borr,
        "bori" => Op::Bori,
        "setr" => Op::Setr,
        "seti" => Op::Seti,
        "gtir" => Op::Gtir,
        "gtri" => Op::Gtri,
        "gtrr" => Op::Gtrr,
        "eqir" => Op::Eqir,
        "eqri" => Op::Eqri,
        "eqrr" => Op::Eqrr,
        _ => { return Err(err!("Not a valid instruction: {}", bits[0])) }
    };
    let inputs = Inputs {
        a: bits[1].parse()?,
        b: bits[2].parse()?,
        c: bits[3].parse()?,
    };

    Ok(Instruction { op, inputs })
}

struct Input {
    pointer_register: usize,
    instructions: Vec<Instruction>
}

mod interpreter {

    #[derive(Debug,Clone)]
    pub struct Interpreter {
        instructions: Vec<Instruction>,
        registers: [usize; 6],
        pointer: usize,
        pointer_register: usize,
    }
    impl Interpreter {
        pub fn new(instructions: Vec<Instruction>, pointer_register: usize) -> Interpreter {
            Interpreter { instructions, pointer_register, pointer: 0, registers: [0;6] }
        }
        pub fn step(&mut self) -> bool {

            if self.pointer >= self.instructions.len() {
                return false;
            }

            let r = &mut self.registers;
            r[self.pointer_register] = self.pointer;
            let Instruction { op, inputs: Inputs{a,b,c}} = self.instructions[self.pointer];
            match op {
                Op::Addr => { r[c] = r[a] + r[b] },
                Op::Addi => { r[c] = r[a] + b },
                Op::Mulr => { r[c] = r[a] * r[b] },
                Op::Muli => { r[c] = r[a] * b },
                Op::Banr => { r[c] = r[a] & r[b] },
                Op::Bani => { r[c] = r[a] & b },
                Op::Borr => { r[c] = r[a] | r[b] },
                Op::Bori => { r[c] = r[a] | b },
                Op::Setr => { r[c] = r[a] },
                Op::Seti => { r[c] = a },
                Op::Gtir => { r[c] = if a > r[b] { 1 } else { 0 } },
                Op::Gtri => { r[c] = if r[a] > b { 1 } else { 0 } },
                Op::Gtrr => { r[c] = if r[a] > r[b] { 1 } else { 0 } },
                Op::Eqir => { r[c] = if a == r[b] { 1 } else { 0 } },
                Op::Eqri => { r[c] = if r[a] == b { 1 } else { 0 } },
                Op::Eqrr => { r[c] = if r[a] == r[b] { 1 } else { 0 } }
            }
            self.pointer = r[self.pointer_register] + 1;
            true
        }
        pub fn registers_mut(&mut self) -> &mut [usize;6] {
            &mut self.registers
        }
        pub fn registers(&self) -> [usize;6] {
            self.registers
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

}