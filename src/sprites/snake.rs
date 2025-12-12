use std::collections::VecDeque;

use tokio::sync::broadcast::Sender;

use crate::sprites::food::Food;

#[derive(PartialEq, Debug)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Clone, Copy)]
pub struct Segment {
    pub x: u16,
    pub y: u16,
}

pub struct Snake {
    pub segments: VecDeque<Segment>,
    direction: Direction,
    _tx: Sender<String>,
}

impl Snake {
    pub fn new(s_count: u16, start_x: u16, start_y: u16, tx: Sender<String>) -> Self {
        let mut segments = VecDeque::new();

        for i in 0..s_count {
            segments.push_back(Segment {
                x: start_x + i,
                y: start_y,
            });
        }

        match tx.send(format!(
            "(unknown-yet) info: initializing snake (length: {}, start_x: {}, start_y: {})",
            s_count, start_x, start_y
        )) {
            Ok(_) => {}
            Err(e) => eprintln!(
                "(unknown-yet) Error: could not broadcast message over tx: {}",
                e
            ),
        }

        Self {
            segments,
            direction: Direction::Right,
            _tx: tx,
        }
    }

    pub fn forward(&mut self, food: &mut Food, width: u16, height: u16) {
        let head = self.segments.back().expect("Snake has no body");

        let (new_x, new_y) = match self.direction {
            Direction::Right => (if head.x < width - 1 { head.x + 1 } else { 0 }, head.y),
            Direction::Left => (if head.x > 0 { head.x - 1 } else { width - 1 }, head.y),
            Direction::Up => (head.x, if head.y > 0 { head.y - 1 } else { height - 1 }),
            Direction::Down => (head.x, if head.y < height - 1 { head.y + 1 } else { 0 }),
        };

        let e_idx = food.eggs.iter().position(|&e| e.x == new_x && e.y == new_y);

        if let Some(e_idx) = e_idx {
            food._replace(e_idx);
            // self._grow(new_x, new_y);
            match self._tx.send(format!(
                "(unknown-yet) info: snake.grow (x: {}, y: {})",
                new_x, new_y
            )) {
                Ok(_) => {}
                Err(e) => eprintln!(
                    "(unknown-yet) Error: could not broadcast message over tx: {}",
                    e
                ),
            }
        }

        self.segments.push_back(Segment { x: new_x, y: new_y });
        self.segments.pop_front();

        match self._tx.send(format!(
            "(unknown-yet) info: snake.forward (x: {}, y: {})",
            new_x, new_y
        )) {
            Ok(_) => {}
            Err(e) => eprintln!(
                "(unknown-yet) Error: could not broadcast message over tx: {}",
                e
            ),
        }
    }

    pub fn turn(&mut self, new_direction: Direction) {
        let opposite = match self.direction {
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        };

        if new_direction != opposite && new_direction != self.direction {
            match self._tx.send(format!(
                "(unknown-yet) info: snake.turn.direction::{:?}",
                new_direction
            )) {
                Ok(_) => {}
                Err(e) => eprintln!(
                    "(unknown-yet) Error: could not broadcast message over tx: {}",
                    e
                ),
            }

            self.direction = new_direction;
        }
    }

    pub fn _grow(&mut self, new_x: u16, new_y: u16) {
        self.segments.push_back(Segment { x: new_x, y: new_y });
    }
}
