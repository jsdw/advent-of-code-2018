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
    // we find the next lowest alpha step to do. What order?
    let mut order = String::new();
    let mut dependencies = deps.clone();
    while let Some(next_step) = find_next_step(&dependencies) {
        remove_step(next_step, &mut dependencies);
        order.push(next_step);
    }
    println!("Star 1: {}", order);

    // For star 2, we have 4 workers completing steps, each
    // which takes a certain time to do. Count up a timer,
    // plucking a task off the dependency list each time a
    // worker becomes free.
    let mut workers: Vec<(char,u32)> = vec![(' ', 0); 5];
    let mut time_spent = 0;
    let mut dependencies = deps;
    while dependencies.len() > 0 {

        // Step forwards as many seconds as we need to do something.
        // Initially, we won't have to wait at all since all workers
        // have 0 seconds left of tasks to do.
        let maybe_wait = workers
            .iter()
            .filter(|(_,t) t != 0)
            .min_by_key(|(_,t)| t)
            .unwrap().1;

        if let Some(wait) = maybe_wait {
            time_spent += wait;
            for w in &mut workers {
                w.1 -= wait;
            }
        }

        // Pluck a task from the dependencies and see how long it
        // will take:
        // * might be step we are working on already! need list of possible steps
        // * so that we can find, at this time step, a step that isn't in progress
        // * to work on if one is available. else we need to hop the next worker to
        // * 0 and try again....
        let next_step = find_next_step(&dependencies).unwrap();
        let next_step_time = next_step as u32 - 4; // A == 61, B == 62..

        // Find a worker ready to do the step:
        let worker = workers.iter_mut().find(|(_,t)| *t == 0).unwrap();

        // remove the step the worker has now finished from our
        // dependencies, and tell the worker to do this new step:
        remove_step(worker.0, &mut dependencies);
        *worker = (next_step,next_step_time);

    }
    println!("Star 2: {}", time_spent);

}

fn find_next_step(dependencies: &HashMap<char,Vec<char>>) -> Option<char> {
    let mut next_steps: Vec<char> = dependencies
        .iter()
        .filter(|(_,v)| v.len() == 0)
        .map(|(&s,_)| s)
        .collect();

    next_steps.sort();
    next_steps.get(0).map(|&c| c)
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