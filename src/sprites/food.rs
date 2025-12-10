use rand::random_range;

#[derive(Copy, Clone, PartialEq)]
pub struct Egg {
    pub x: u16,
    pub y: u16,
}

pub struct Food {
    _width: u16,
    _height: u16,
    pub eggs: Vec<Egg>,
}

impl Food {
    pub fn new(count: u16, width: u16, height: u16) -> Food {
        let mut eggs = Vec::new();
        for _ in 0..count {
            let egg = Egg {
                x: random_range(0..width),
                y: random_range(0..height),
            };
            eggs.push(egg);
        }
        Food {
            eggs,
            _width: width,
            _height: height,
        }
    }

    pub fn _replace(&mut self, idx: usize) {
        self.eggs[idx] = Egg {
            x: random_range(0..self._width),
            y: random_range(0..self._height),
        };
    }
}
