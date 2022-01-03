mod command;
mod direction;
mod game;
mod snake;
mod vector;
use crate::game::run_game;
use crate::game::Game;
use std::io::stdout;

fn main() {
    run_game(&mut Game::new(stdout(), 10, 10));
}
