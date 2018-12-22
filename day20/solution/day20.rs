use crate::regex::{ Regex, Direction };
// use crate::list::List;
use std::error::Error;
use std::result;
use std::collections::HashMap;

macro_rules! err { ($($tt:tt)*) => { Box::<$crate::Error>::from(format!($($tt)*)) } }
pub type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: String = std::fs::read_to_string(filename).expect("can't open file");

    let re = Regex::new(&input)?;
    assert_eq!(re.to_string(), input); // confirm that our parsing is good.

    // Count up distances to each room:
    let mut distances = HashMap::new();
    steps(&re, vec![(0,0)], &mut |(ox,oy), (nx,ny)| {
        let last_dist = *distances.get(&(ox,oy)).unwrap_or(&0);
        distances.entry((nx,ny)).or_insert(last_dist+1);
    });

    let furthest = distances.values().max().unwrap();
    println!("Star 1: {}", furthest);

    let distant = distances.values().filter(|&&v| v >= 1000).count();
    println!("Star 2: {}", distant);

    Ok(())
}

fn steps(re: &Regex, mut tails: Vec<(i64,i64)>, func: &mut impl FnMut((i64,i64), (i64,i64))) -> Vec<(i64,i64)> {
    tails.sort_unstable();
    tails.dedup();
    match re {
        Regex::And(regexs) => {
            for re in regexs {
                tails = steps(re, tails, func);
            }
            tails
        },
        Regex::Or(regexs, is_optional) => {
            let mut all_paths = if *is_optional {
                tails.clone()
            } else {
                vec![]
            };
            for re in regexs {
                all_paths.append(&mut steps(re, tails.clone(), func));
            }
            all_paths
        },
        Regex::Value(values) => {
            for tail in &mut tails {
                for val in values {
                    let (x,y) = *tail;
                    let new_tail = match val {
                        Direction::N => (x, y-2),
                        Direction::E => (x+2, y),
                        Direction::S => (x, y+2),
                        Direction::W => (x-2, y)
                    };
                    func((x,y), new_tail);
                    *tail = new_tail;
                }
            }
            tails
        }
    }
}

mod regex {

    use super::Result;

    #[derive(Debug,PartialEq,Eq,Clone)]
    pub enum Regex {
        And(Vec<Regex>),
        Or(Vec<Regex>, bool),
        Value(Vec<Direction>)
    }

    impl Regex {
        pub fn new(s: &str) -> Result<Regex> {
            parse_regex(s.as_bytes())
        }
        pub fn to_string(&self) -> String {
            let mut s = String::new();
            s.push('^');
            stringify_regex(self, &mut s);
            s.push('$');
            s
        }
    }

    #[derive(Debug,PartialEq,Eq,Clone,Copy)]
    pub enum Direction {
        N, E, S, W
    }

    fn parse_regex(s: &[u8]) -> Result<Regex> {
        if !s.starts_with(&[b'^']) {
            Err(err!("Should begin with '^'"))
        } else if !s.ends_with(&[b'$']) {
            Err(err!("Should end with '$'"))
        } else {
            let (val, _) = parse_regex_and(&s[1..s.len()-1])?;
            Ok(val)
        }
    }

    fn parse_regex_and(mut s: &[u8]) -> Result<(Regex,&[u8])> {
        let mut bits = vec![];
        while let Some(c) = s.first() {
            match c {
                b'|' | b')' => {
                    break; // end of the AND; break.
                },
                b'(' => {
                    let (val, rest) = parse_regex_or(s)?;
                    bits.push(val);
                    s = rest;
                },
                _ => {
                    let (val, rest) = parse_regex_value(s)?;
                    bits.push(val);
                    s = rest;
                }
            }
        }
        Ok((Regex::And(bits), s))
    }

    fn parse_regex_or(mut s: &[u8]) -> Result<(Regex,&[u8])> {
        let mut bits = vec![];
        let mut is_optional = false;
        s = &s[1..]; // ignore first (
        while let Some(c) = s.first() {
            match c {
                b')' => {
                    s = &s[1..];
                    break; // end of the OR; break.
                },
                b'|' => {
                    s = &s[1..];
                    if let Some(b')') = s.first() {
                        is_optional = true;
                    }
                },
                _ => {
                    let (val, rest) = parse_regex_and(s)?;
                    bits.push(val);
                    s = rest;
                }
            }
        }
        Ok((Regex::Or(bits, is_optional), s))
    }

    fn parse_regex_value(mut s: &[u8]) -> Result<(Regex,&[u8])> {
        let mut current = vec![];
        while let Some(c) = s.first() {
            match c {
                b'(' | b'|' | b')' => {
                    break; // end of the value; break.
                },
                _ => {
                    current.push(parse_direction(*c)?);
                    s = &s[1..];
                }
            }
        }
        Ok((Regex::Value(current),s))
    }

    fn parse_direction(c: u8) -> Result<Direction> {
        match c {
            b'N' => Ok(Direction::N),
            b'E' => Ok(Direction::E),
            b'S' => Ok(Direction::S),
            b'W' => Ok(Direction::W),
            _ => Err(err!("'{}' is not a valid Direction", c as char))
        }
    }

    fn stringify_regex(re: &Regex, buf: &mut String) {
        match re {
            Regex::And(rs) => {
                for r in rs {
                    stringify_regex(r, buf);
                }
            },
            Regex::Or(rs, is_optional) => {
                buf.push('(');
                for (i,r) in rs.iter().enumerate() {
                    stringify_regex(r, buf);
                    if *is_optional || i < rs.len()-1 { buf.push('|') }
                }
                buf.push(')');
            },
            Regex::Value(ds) => {
                for d in ds {
                    stringify_direction(d, buf);
                }
            }
        }
    }

    fn stringify_direction(dir: &Direction, buf: &mut String) {
        match dir {
            Direction::N => { buf.push('N') },
            Direction::E => { buf.push('E') },
            Direction::S => { buf.push('S') },
            Direction::W => { buf.push('W') },
        }
    }

}