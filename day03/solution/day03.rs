use crate::{square::Square, canvas::Canvas};
use std::collections::HashSet;

fn main() {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let squares: Vec<(usize,Square)> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .lines()
        .map(square::parse)
        .collect();

    // This is what we'll draw onto the Canvas, to keep
    // track of the ID responsible for each pixel, or
    // whether the pixel is already overlapping:
    #[derive(Clone,Copy,PartialEq,Eq)]
    pub enum Pixel {
        Value(usize),
        Overlapping
    }

    // Draw our pixels onto the canvas, merging each
    // with the current value drawn and keeping track
    // of overlaps for part 2:
    let mut canvas = Canvas::new(1000,1000);
    let mut overlapping = HashSet::new();
    for &(value,s) in &squares {
        canvas.draw(&s, |current_value| {
            Some(match current_value {
                None => {
                    Pixel::Value(value)
                },
                Some(Pixel::Overlapping) => {
                    overlapping.insert(value);
                    Pixel::Overlapping
                },
                Some(Pixel::Value(current_value)) => {
                    overlapping.insert(current_value);
                    overlapping.insert(value);
                    Pixel::Overlapping
                }
            })
        })
    }

    // Count overlaps:
    let mut num_overlapping = 0;
    for v in canvas.values() {
        if let Some(Pixel::Overlapping) = v {
            num_overlapping += 1;
        }
    }
    println!("Star 1: {}", num_overlapping);

    // Find any ID that did not overlap:
    let not_overlapped = squares
        .iter()
        .find(|(value,_)| !overlapping.contains(value))
        .map(|(value,_)| value);
    println!("Star 2: {:?}", not_overlapped);

}

/// Defining and parsing Squares:
mod square {
    use regex::Regex;
    use lazy_static::lazy_static;

    #[derive(Copy,Clone,Debug)]
    pub struct Square {
        pub left: usize,
        pub top: usize,
        pub width: usize,
        pub height: usize
    }

    pub fn parse(s: &str) -> (usize, Square) {
        lazy_static! {
            static ref re: Regex = Regex::new(r"#(\d+) @ (\d+),(\d+): (\d+)x(\d+)").unwrap();
        }
        let caps = re.captures(s.trim()).unwrap();
        let get = |n| caps.get(n).unwrap().as_str().parse().unwrap();
        (get(1), Square { left: get(2), top: get(3), width: get(4), height: get(5) })
    }

}

/// A Canvas to draw squares onto:
mod canvas {
    use crate::square;

    pub struct Canvas<T> {
        width: usize,
        height: usize,
        inner: Vec<Option<T>>,
    }

    impl <T: Clone> Canvas<T> {
        pub fn new(width: usize, height: usize) -> Canvas<T> {
            Canvas {
                width: width,
                height: height,
                inner: vec![None; width * height]
            }
        }
        pub fn draw<F>(&mut self, square: &square::Square, mut merge: F)
        where F: FnMut(Option<T>) -> Option<T> {
            let right = self.width.min(square.left+square.width);
            let bottom = self.height.min(square.top+square.height);
            for x in square.left .. right {
                for y in square.top .. bottom {
                    let idx = y * self.width + x;
                    self.inner[idx] = merge(self.inner[idx].clone());
                }
            }
        }
        pub fn values(&self) -> impl Iterator<Item=Option<T>> + '_ {
            self.inner.iter().cloned()
        }
    }

}