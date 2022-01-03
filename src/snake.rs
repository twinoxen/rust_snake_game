use crate::vector::*;
use crate::direction::*;

#[derive(Debug)]
pub struct Snake {
    body: Vec<Vector>,
    direction: Direction,
    digesting: bool,
}

impl Snake {
    pub fn new(start: Vector, length: u16, direction: Direction) -> Self {
        let opposite = opposite_direction(direction);

        let body: Vec<Vector> = (0..length)
            .into_iter()
            .map(|i| move_vector_to(&start, opposite, i.try_into().unwrap()))
            .collect();

        Self {
            body,
            direction,
            digesting: false,
        }
    }
}

pub fn get_head_point(snake: &Snake) -> Vector {
    snake.body.first().unwrap().clone()
}

pub fn get_body_point(snake: &Snake) -> Vec<Vector> {
    snake.body.clone()
}

pub fn get_direction(snake: &Snake) -> Direction {
    snake.direction.clone()
}

pub fn contains_point(snake: &Snake, vector: &Vector) -> bool {
    snake.body.contains(vector)
}

pub fn slither(snake: &mut Snake) -> &Snake {
    snake.body.insert(
        0,
        move_vector_to(snake.body.first().unwrap(), snake.direction, 1),
    );

    if !snake.digesting {
        snake.body.remove(snake.body.len() - 1);
    } else {
        snake.digesting = false;
    }

    snake
}

pub fn set_direction(snake: &mut Snake, direction: Direction) -> &Snake {
    snake.direction = direction;

    snake
}

pub fn grow(snake: &mut Snake) -> &Snake {
    snake.digesting = true;

    snake
}