use lazy_static::lazy_static;
use regex::Regex;

use std::i64;

fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let mut points: Vec<_> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(parse_point)
        .collect();

    println!("The sky says:");
    let mut within_range = false;
    for seconds in 1.. {
        // move each point by its velocity:
        for p in &mut points {
            p.position.0 += p.velocity.0;
            p.position.1 += p.velocity.1;
        }

        // print them if they appear to be pretty close. End the loop
        // if they pass by and then stop being close:
        let Bounds { top, bottom, left, right } = calculate_bounds(&points);
        if bottom - top < 15 && right - left < 100 {
            within_range = true;
            print_points(&points);
            println!("This will appear in {} seconds", seconds);
        } else if within_range {
            break;
        }
    }

}

fn parse_point(s: &str) -> Point {
    lazy_static! {
        static ref point_re: Regex = Regex::new(r"position=<\s*(-?\d+),\s*(-?\d+)> velocity=<\s*(-?\d+)\s*,\s*(-?\d+)\s*>").unwrap();
    }
    let caps = point_re.captures(s).expect("expects a proper point");
    let get = |n| caps.get(n).unwrap().as_str().parse().expect("expected valid integer");

    Point {
        position: (get(1), get(2)),
        velocity: (get(3), get(4))
    }
}

fn calculate_bounds(points: &[Point]) -> Bounds {
    let mut top = i64::MAX;
    let mut bottom = i64::MIN;
    let mut left = i64::MAX;
    let mut right = i64::MIN;
    for (x,y) in points.iter().map(|p| p.position) {
        top = i64::min(top, y);
        bottom = i64::max(bottom, y);
        left = i64::min(left, x);
        right = i64::max(right, x);
    }
    Bounds { top, bottom, left, right }
}

fn print_points(points: &[Point]) {
    let Bounds { top, bottom, left, right } = calculate_bounds(points);
    let height = bottom - top + 1;
    let width = right - left + 1;
    let mut canvas = vec![false; height as usize * width as usize];
    for (x,y) in points.iter().map(|p| p.position) {
        let pos = (y-top) * width + (x-left);
        canvas[pos as usize] = true;
    }

    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for row in canvas.chunks(width as usize) {
        let row: Vec<u8> = row
            .iter()
            .map(|&b| if b { b'#' } else { b'.' })
            .collect();

        handle.write_all(&row).expect("failed to write row");
        handle.write_all(&[b'\n']).expect("failed to write row end");
    }
}

#[derive(Debug)]
struct Point {
    position: (i64,i64),
    velocity: (i64,i64)
}

#[derive(Debug)]
struct Bounds {
    top: i64,
    bottom: i64,
    left: i64,
    right: i64
}