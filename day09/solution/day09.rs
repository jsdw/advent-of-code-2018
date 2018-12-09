use self::ring::Ring;
use std::collections::HashMap;

fn main() {
    println!("Star 1: {}", play_marbles(405, 71700));
    println!("Star 2: {}", play_marbles(405, 7170000));
}

fn play_marbles(players: usize, last_points: u128) -> u128 {
    let mut marbles = Ring::new(0);
    let mut scores = HashMap::new();

    for (player,n) in (0..players).cycle().zip(0u128..last_points).skip(1) {
        // every 23 moves, add points to the current player:
        if n != 0 && n % 23 == 0 {
            let points = marbles.backward(6).remove_before().expect("marble expected") + n;
            *scores.entry(player).or_insert(0) += points;
        }
        // else, go round one marble, add the next marble after it, then go to it:
        else {
            marbles.next().insert_after(n).next();
        }
        // end when the marble we're on equals the last_points value given:
        if *marbles.value() == last_points {
            break;
        }
    }

    // return the best score an elf got:
    *scores.values().max().unwrap()
}

// This module implements a basic ring structure for inserting and removing from
// our ring of marbles. We only add the functionality we need to do the puzzle:
mod ring {

    #[derive(Debug)]
    pub struct Ring<T> {
        free: Vec<usize>,
        nodes: Vec<Node<T>>,
        idx: usize
    }

    #[derive(Debug)]
    struct Node<T> {
        next: usize,
        prev: usize,
        value: Option<T>
    }

    impl <T> Ring<T> {
        pub fn new(value: T) -> Ring<T> {
            Ring {
                nodes: vec![Node {
                    next: 0,
                    prev: 0,
                    value: Some(value)
                }],
                free: vec![],
                idx: 0
            }
        }
        pub fn insert_after(&mut self, value: T) -> &mut Self {
            let next_idx = self.nodes[self.idx].next;
            let new_node = Node {
                next: self.nodes[self.idx].next,
                prev: self.idx,
                value: Some(value)
            };

            let new_idx = if let Some(idx) = self.free.pop() {
                self.nodes[idx] = new_node;
                idx
            } else {
                self.nodes.push(new_node);
                self.nodes.len() - 1
            };

            self.nodes[self.idx].next = new_idx;
            self.nodes[next_idx].prev = new_idx;
            self
        }
        pub fn remove_before(&mut self) -> Option<T> {
            let prev_idx = self.nodes[self.idx].prev;
            if prev_idx == self.idx {
                panic!("Cannot remove the last element");
            }

            let before_idx = self.nodes[prev_idx].prev;

            self.nodes[self.idx].prev = before_idx;
            self.nodes[before_idx].next = self.idx;

            self.free.push(prev_idx);
            self.nodes[prev_idx].value.take()
        }
        pub fn backward(&mut self, n: usize) -> &mut Self {
            (0..n).for_each(|_| { self.prev(); });
            self
        }
        pub fn next(&mut self) -> &mut Self {
            self.idx = self.nodes[self.idx].next;
            self
        }
        pub fn prev(&mut self) -> &mut Self {
            self.idx = self.nodes[self.idx].prev;
            self
        }
        pub fn value(&self) -> &T {
            self.nodes[self.idx].value.as_ref().unwrap()
        }
    }

}