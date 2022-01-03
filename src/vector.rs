use crate::direction::*;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Vector {
    pub x: i16,
    pub y: i16,
}

impl Vector {
    pub fn new(x: i16, y: i16) -> Self {
        let x = if x < 0 { 0 } else { x };
        let y = if y < 0 { 0 } else { y };

        Self { x, y }
    }
}

pub fn move_vector_to(vector: &Vector, direction: Direction, step: i16) -> Vector {
    let x = vector.x;
    let y = vector.y;

    let move_direction_by_step = match direction {
        Direction::Up => (0, y - step),
        Direction::Right => (x + step, 0),
        Direction::Down => (0, y + step),
        Direction::Left => (x - step, 0),
    };

    Vector::new(move_direction_by_step.0, move_direction_by_step.1)
}
