use std::collections::{ HashMap };
use regex::Regex;
use lazy_static::lazy_static;

fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: Vec<(char, char)> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_dependency)
        .collect();

    // Create a map from step to steps it depends on, inserting dependencies
    // as steps as well so that we don't forget to do them too.
    let deps = input.into_iter().fold(HashMap::new(), |mut m, (dependency,step)| {
        m.entry(step).or_insert(vec![]).push(dependency);
        m.entry(dependency).or_insert(vec![]);
        m
    });

    // For star 1, we can do one step at a time. Each time we do it,
    // we find the next lowest alpha step to do.
    let mut order = String::new();
    let mut dependencies = deps.clone();
    while dependencies.len() > 0 {
        let next_step = *find_next_steps(&dependencies).get(0).unwrap();
        remove_step(next_step, &mut dependencies);
        order.push(next_step);
    }
    println!("Star 1: {}", order);

    // For star 2, we have 5 workers completing steps, as fast as
    // possible (in parallel), each which takes a certain time to do.
    let mut workers = vec![(None, 0); 5];
    let mut pending_steps: Vec<char> = vec![];
    let mut time_spent = 0;
    let mut dependencies = deps;
    while dependencies.len() > 0 {
        // How far do we need to step forwards in
        // time until we can potentially act?
        let wait = workers
            .iter()
            .map(|(_,t)| *t)
            .filter(|t| *t != 0)
            .min()
            .unwrap_or(0);

        // Step forward, "finishing" the step for
        // the worker that hits 0.
        time_spent += wait;
        for w in &mut workers {
            if w.1 != 0 { w.1 -= wait }
            if w.1 == 0 && w.0.is_some() {
                remove_step(w.0.unwrap(), &mut dependencies);
                w.0 = None;
            }
        }

        // Get any steps that are currently available and
        // merge them with any existing pending steps, ignoring
        // any that are already in progress:
        let mut next_steps = find_next_steps(&dependencies)
            .into_iter()
            .filter(|&c| workers.iter().filter_map(|(s,_)| *s).all(|s| c != s))
            .collect();

        pending_steps.append(&mut next_steps);
        pending_steps.sort();
        pending_steps.dedup();

        // Assign as many pending steps to workers as possible
        // at this time step, for maximum efficiency:
        for w in workers.iter_mut().filter(|(_,t)| *t == 0) {
            if pending_steps.len() == 0 { break };
            let next_step = pending_steps.remove(0);
            let next_step_time = next_step as u32 - 4; // A == 61, B == 62..
            *w = (Some(next_step),next_step_time);
        }
    }
    println!("Star 2: {}", time_spent);

}

fn find_next_steps(dependencies: &HashMap<char,Vec<char>>) -> Vec<char> {
    let mut next_steps: Vec<char> = dependencies
        .iter()
        .filter(|(_,v)| v.len() == 0)
        .map(|(&s,_)| s)
        .collect();

    next_steps.sort();
    next_steps
}

fn remove_step(step: char, dependencies: &mut HashMap<char,Vec<char>>) {
    dependencies.remove(&step);

    // remove this dep from other steps:
    for v in dependencies.values_mut() {
        v.retain(|&c| c != step)
    }
}

fn parse_dependency(s: &str) -> (char, char) {
    lazy_static! {
        static ref dep_re: Regex = Regex::new(r"Step ([A-Z]) must be finished before step ([A-Z]) can begin.").unwrap();
    }
    let caps = dep_re.captures(s.trim()).unwrap();
    let get = |n| caps.get(n).unwrap().as_str().chars().next().unwrap();
    (get(1), get(2))
}