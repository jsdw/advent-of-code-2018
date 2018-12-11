use self::vec2d::Vec2D;

fn main() {

    let serial_no = 7857;

    let mut cells = Vec2D::new(299,299,0);
    for x in 1..300 {
        for y in 1..300 {
            cells.set(x-1, y-1, compute_cell(serial_no, x,y));
        }
    }

    let mut best_x = 0;
    let mut best_y = 0;
    let mut best_sum = 0;
    for x in 1..300-2 {
        for y in 1..300-2 {
            let sum = cells.values_sub(x-1, y-1, 3, 3).sum();
            if sum > best_sum {
                best_x = x;
                best_y = y;
                best_sum = sum;
            }
        }
    }
    println!("Star 1: {},{}", best_x, best_y);

    let mut best_x = 0;
    let mut best_y = 0;
    let mut best_sum = 0;
    let mut best_size = 0;
    for size in 1..=300 {
        for x in 1..300-size {
            for y in 1..300-size {
                let sum = cells.values_sub(x-1, y-1, size,size).sum();
                if sum > best_sum {
                    best_x = x;
                    best_y = y;
                    best_sum = sum;
                    best_size = size;
                }
            }
        }
    }
    println!("Star 1: {},{},{}", best_x, best_y, best_size);

}

fn compute_cell(serial_no: usize, x: usize, y: usize) -> i32 {
    let rack_id = x + 10;
    let power_level = rack_id * y + serial_no;
    let power_level = power_level * rack_id;
    let hundreds = power_level
        .to_string()
        .bytes()
        .rev()
        .nth(2)
        .map(|c| c - 48)
        .unwrap_or(0);
    hundreds as i32 - 5
}

// A quick 2D vector which can return iterators over 2D sub-ranges:
mod vec2d {

    #[derive(Clone,Debug)]
    pub struct Vec2D<T> {
        width: usize,
        height: usize,
        values: Vec<T>
    }

    impl <T: Clone> Vec2D<T> {
        pub fn new(width: usize, height: usize, value: T) -> Vec2D<T> {
            Vec2D {
                width: width,
                height: height,
                values: vec![value; width * height]
            }
        }
        pub fn set(&mut self, x: usize, y: usize, value: T) {
            let idx = y * self.width + x;
            self.values[idx] = value;
        }
        pub fn values_sub(&self, x: usize, y: usize, width: usize, height: usize) -> impl Iterator<Item=&T> {
            (y..y+height).flat_map(move |y| {
                (x..x+width).map(move |x| {
                    &self.values[y * self.width + x]
                })
            })
        }
    }

}