use self::vec2d::Vec2D;

fn main() {
    let serial_no = 7857;
    let mut cells = Vec2D::new(300,300,0);
    for x in 1..=300 {
        for y in 1..=300 {
            cells.set(x-1, y-1, compute_cell(serial_no, x,y));
        }
    }

    // Star 1: Find best location for 3x3 grid:
    let (x, y, _) = best_cell_of_size(&cells, 3);
    println!("Star 1: {},{}", x, y);

    // Star 2: Find best location for any grid of size 1..=300:
    let (x, y, size, _) = (1..=300)
        .map(|size| {
            let (x, y, sum) = best_cell_of_size(&cells, size);
            (x, y, size, sum)
        })
        .max_by_key(|(_,_,_,sum)| *sum)
        .unwrap();
    println!("Star 2: {},{},{}", x, y, size);
}

fn best_cell_of_size(cells: &Vec2D<i32>, size: usize) -> (usize,usize,i32) {
    let sums =
        (0 ..= cells.width() - size).flat_map(|x| {
            (0 ..= cells.height() - size).map(move |y| {
                let sum = cells.values_sub(x, y, size, size).sum();
                (x+1, y+1, sum) // go from 0 to 1 indexed
            })
        });
    sums.max_by_key(|(_x,_y,sum)| *sum).unwrap()
}

fn compute_cell(serial_no: usize, x: usize, y: usize) -> i32 {
    let rack_id = x + 10;
    let power_level = rack_id * y + serial_no;
    let power_level = power_level * rack_id;
    let hundreds = (power_level / 100) % 10;
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

    impl <T> Vec2D<T> {
        pub fn new(width: usize, height: usize, value: T) -> Vec2D<T> where T: Clone {
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
        pub fn width(&self) -> usize {
            self.width
        }
        pub fn height(&self) -> usize {
            self.height
        }
    }
}