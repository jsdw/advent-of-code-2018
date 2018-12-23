use std::error::Error;
use std::result;
use std::collections::{ HashSet, HashMap };

fn main() {
    println!("Star 1: {}", star1());
    println!("Star 2: {}", star2());
}

// Disassemble and run the instructions given
// for one loop, to see what value r0 needs to
// be to equal r4 and break out of the loop:
fn star1() -> usize {
    let mut r4 = 0;
    let mut r3 = r4 | 65536;
    r4 = 3730679;

    loop {
        r4 = r4 + (r3 & 255);
        r4 = r4 & 16777215;
        r4 = r4 * 65899;
        r4 = r4 & 16777215;

        if 256 > r3 {
            break;
        }
        
        r3 = r3 / 256; // integer division floors
    }
    r4
}

fn star2() -> usize {
    let mut results = HashMap::new();
    let mut counter = 0;
    let mut r4 = 0;

    loop {
        counter += 1;
        let mut r3 = r4 | 65536;
        r4 = 3730679;

        loop {
            counter += 1;
            r4 = r4 + (r3 & 255);
            r4 = r4 & 16777215;
            r4 = r4 * 65899;
            r4 = r4 & 16777215;

            if 256 > r3 {
                break;
            }
            
            r3 = r3 / 256; // integer division floors
        }

        // Track all of the possible values for r4
        // along with relatively how many instructions
        // have run to get this far (counter). Break
        // If we run into the same r4 again since we'll
        // then loop forever. r0 can equal any of these
        // to casue the program to break out of the loop.
        if results.contains_key(&r4) {
            break;
        }
        results.insert(r4, counter);

    };

    // Which r4 had roughly the highest
    // number of instructions:
    results.into_iter().max_by_key(|(k,v)| *v).unwrap().0
}