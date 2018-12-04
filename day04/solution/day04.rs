use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{Ordering};
use std::collections::HashMap;
use self::EventType::*;

fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let mut events: Vec<_> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_event)
        .collect();

    // Sort events chronologically:
    events.sort_by_key(|ev| ev.date);

    let best_guard_id = 0;
    let best_time_asleep = 0;
    let current_guard_id = 0;
    let current_time_asleep = 0;
    let current_hour = 0;
    let current_minute = 0;

    for e in &events {
        match e.ty {
            GuardBegins(id) => {
                if current_time_asleep > best_time_asleep {
                    best_guard_id = current_guard_id;
                    best_time_asleep = current_time_asleep;
                }
                current_guard_id = id;
                current_hour = e.date.hour;
                current_minute = e.date.minute;
                current_time_asleep = 0;
            },
            FallsAsleep => {

            },
            WakesUp => {

            }
        }
    }
    println!("Star 1: {}", best_guard_id as u32 * best_time_asleep);

    for e in events {
        println!("{:?}", e);
    }

}

fn parse_event(s: &str) -> Event {
    lazy_static! {
        static ref datetime_re: Regex = Regex::new(r"\[1518-(\d\d)-(\d\d) (\d\d):(\d\d)\]").unwrap();
        static ref begins_shift_re: Regex = Regex::new(r"Guard #(\d+) begins shift").unwrap();
        static ref wakes_up_re: Regex = Regex::new(r"wakes up").unwrap();
        static ref falls_asleep_re: Regex = Regex::new(r"falls asleep").unwrap();
    }
    let s = s.trim();

    let dt = datetime_re.captures(s).expect("expects a proper date");
    let month = dt.get(1).unwrap().as_str().parse().unwrap();
    let day = dt.get(2).unwrap().as_str().parse().unwrap();
    let hour = dt.get(3).unwrap().as_str().parse().unwrap();
    let minute = dt.get(4).unwrap().as_str().parse().unwrap();

    let ty = if wakes_up_re.is_match(s) {
        EventType::WakesUp
    } else if falls_asleep_re.is_match(s) {
        EventType::FallsAsleep
    } else if let Some(caps) = begins_shift_re.captures(s) {
        let id = caps.get(1).unwrap().as_str().parse().unwrap();
        EventType::GuardBegins(id)
    } else {
        panic!("Could not parse input: {}", s);
    };

    Event {
        date: DateTime { month, day, hour, minute },
        ty
    }
}

#[derive(Eq,PartialEq,Debug,Clone,Copy)]
struct Event {
    date: DateTime,
    ty: EventType
}

#[derive(Eq,PartialEq,Debug,Clone,Copy)]
enum EventType {
    WakesUp,
    FallsAsleep,
    GuardBegins(u16)
}

#[derive(Eq,PartialEq,Debug,Clone,Copy,PartialOrd,Ord)]
struct DateTime {
    month: u8,
    day: u8,
    hour: u8,
    minute: u8
}

impl DateTime {
    /// Distance between the hour and minute of two datetimes, always positive.
    fn time_distance(&self, other: &DateTime) -> u32 {
        let (small,big) = if (self.hour,self.minute) < (other.hour,other.minute) {
            (self,other)
        } else {
            (other,self)
        };

        let mut diff = (other.hour - self.hour) as u32 * 60;

        if small.minute < big.minute {
            diff += (big.minute - small.minute) as u32;
        } else {
            diff -= (small.minute - big.minute) as u32;
        }

        diff
    }
}
