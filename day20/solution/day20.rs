use crate::regex::Regex;
use std::error::Error;
use std::result;

macro_rules! err { ($($tt:tt)*) => { Box::<$crate::Error>::from(format!($($tt)*)) } }
pub type Result<T> = result::Result<T, Box<dyn Error + 'static>>;

fn main() -> Result<()> {
    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: String = std::fs::read_to_string(filename).expect("can't open file");

    let re = Regex::new(&input)?;
    assert_eq!(re.to_string(), input); // confirm that our parsing is good.

    Ok(())
}

mod regex {

    use super::Result;

    #[derive(Debug)]
    pub enum Regex {
        And(Vec<Regex>),
        Or(Vec<Regex>),
        Optional(Box<Regex>),
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

    #[derive(Debug)]
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
                    let (val, rest) = parse_regex_optional(s)?;
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
        let res = if bits.len() == 1 {
            bits.remove(0)
        } else {
            Regex::And(bits)
        };
        Ok((res, s))
    }

    fn parse_regex_or(mut s: &[u8]) -> Result<(Regex,&[u8])> {
        let mut bits = vec![];
        while let Some(c) = s.first() {
            match c {
                b')' => {
                    break; // end of the OR; break.
                },
                b'|' => {
                    s = &s[1..];
                },
                _ => {
                    let (val, rest) = parse_regex_and(s)?;
                    bits.push(val);
                    s = rest;
                }
            }
        }
        Ok((Regex::Or(bits), s))
    }

    fn parse_regex_optional(s: &[u8]) -> Result<(Regex,&[u8])> {
        if s.len() < 2 {
            Err(err!("Expecting at least '(' and ')' but not enough input"))
        } else if s[0] != b'(' {
            Err(err!("Optional expecting to start with '('"))
        } else {
            let (val, rest) = parse_regex_or(&s[1..])?;
            if rest[0] != b')' {
                Err(err!("Optional expecting to end with ')', got '{}'", rest[0] as char))
            } else {
                Ok((Regex::Optional(Box::new(val)),&rest[1..]))
            }
        }
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
            Regex::Or(rs) => {
                let needs_pipe = rs.len() == 1;
                for (i,r) in rs.iter().enumerate() {
                    stringify_regex(r, buf);
                    if needs_pipe || i < rs.len()-1 { buf.push('|') }
                }
            },
            Regex::Optional(r) => {
                buf.push('(');
                stringify_regex(r, buf);
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