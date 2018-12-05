use lazy_static::lazy_static;
use regex::Regex;
use self::EventType::*;
use std::collections::HashMap;

fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let mut events: Vec<_> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_event)
        .collect();

    // Sort events chronologically:
    events.sort_by_key(|ev| (ev.date, ev.time));

    // Tally up how long each guard spent each minute asleep:
    let sleep_times = tally_slept_minutes(&events);

    // Get our answers from this data:
    println!("Star 1: {}", day1(&sleep_times));
    println!("Star 2: {}", day2(&sleep_times));

}

fn day1(sleep_times: &HashMap<u16, [u32;60]>) -> usize {
    // Which guard slept the most in total and which minutes?
    let (&sleepiest_guard_id, &sleepiest_guard_minutes) = sleep_times
        .iter()
        .max_by_key(|&(_,times)| times.iter().sum::<u32>())
        .unwrap();

    // Look to see which minute this guard was asleep the most:
    let sleepiest_minute = sleepiest_guard_minutes
        .iter()
        .enumerate()
        .max_by_key(|&(_,m)| m)
        .unwrap().0;

    sleepiest_minute * sleepiest_guard_id as usize
}

fn day2(sleep_times: &HashMap<u16, [u32;60]>) -> usize {
    // Find out which guard spent a single minute asleep the most:
    let (sleepiest_guard_id, sleepiest_guard_minute, _) = sleep_times
        .iter()
        .map(|(&id,times)| {
            let (min,&time) = times.into_iter().enumerate().max_by_key(|&(_,v)| v).unwrap();
            (id, min, time)
        })
        .max_by_key(|&(_,_,time)| time)
        .unwrap();

    sleepiest_guard_minute * sleepiest_guard_id as usize
}

fn tally_slept_minutes(events: &Vec<Event>) -> HashMap<u16, [u32;60]> {
    // Tally up how long each guard spent each minute asleep:
    let mut sleep_times = HashMap::new();
    let mut current_guard_id = 0;
    let mut started_sleeping_min = 0;
    for e in events {
        match e.ty {
            GuardBegins(id) => {
                current_guard_id = id;
            },
            FallsAsleep => {
                started_sleeping_min = e.time.minute;
            },
            WakesUp => {
                let awake_min = e.time.minute;
                let mins_slept = sleep_times.entry(current_guard_id).or_insert([0u32; 60]);
                for v in &mut mins_slept[started_sleeping_min as usize .. awake_min as usize] {
                    *v += 1;
                }
            }
        }
    }
    sleep_times
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
        date: Date { month, day },
        time: Time { hour, minute },
        ty
    }
}

#[derive(Eq,PartialEq,Debug,Clone,Copy)]
struct Event {
    date: Date,
    time: Time,
    ty: EventType
}

#[derive(Eq,PartialEq,Debug,Clone,Copy)]
enum EventType {
    WakesUp,
    FallsAsleep,
    GuardBegins(u16)
}

#[derive(Eq,PartialEq,Debug,Clone,Copy,PartialOrd,Ord)]
struct Date {
    month: u8,
    day: u8
}

#[derive(Eq,PartialEq,Debug,Clone,Copy,PartialOrd,Ord)]
struct Time {
    hour: u8,
    minute: u8
}
