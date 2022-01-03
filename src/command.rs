use crate::direction::*;
pub enum Command {
  Quit,
  Turn(Direction),
}