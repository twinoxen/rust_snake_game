use crate::command::*;
use crate::direction::*;
use crate::snake;
use crate::vector::*;
use crossterm::cursor::Hide;
use crossterm::cursor::MoveTo;
use crossterm::cursor::Show;
use crossterm::event::*;
use crossterm::style::*;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::size;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::SetSize;
use crossterm::ExecutableCommand;
use rand::Rng;
use std::io::Stdout;
use std::time::{Duration, Instant};

const MAX_INTERVAL: u16 = 700;
const MIN_INTERVAL: u16 = 200;
const MAX_SPEED: u16 = 20;

#[derive(Debug)]
pub struct Game {
    stdout: Stdout,
    original_terminal_size: (u16, u16),
    width: u16,
    height: u16,
    food: Option<Vector>,
    snake: snake::Snake,
    speed: u16,
    score: u16,
}

impl Game {
    pub fn new(stdout: Stdout, width: u16, height: u16) -> Self {
        let original_terminal_size: (u16, u16) = size().unwrap();

        let i_width = i16::try_from(width).expect("Can't have a negative width!");
        let i_height = i16::try_from(height).expect("Can't have a negative height!");

        Self {
            stdout,
            original_terminal_size,
            width,
            height,
            food: None,
            snake: snake::Snake::new(
                Vector::new(i_width / 2, i_height / 2),
                3,
                match rand::thread_rng().gen_range(0, 4) {
                    0 => Direction::Up,
                    1 => Direction::Right,
                    2 => Direction::Down,
                    _ => Direction::Left,
                },
            ),
            speed: 0,
            score: 0,
        }
    }
}

pub fn run_game(game: &mut Game) {
    place_food(game);
    prepare_ui(game);
    render(game);

    let mut done = false;
    while !done {
        let interval = calculate_interval(game);
        let direction = snake::get_direction(&game.snake);
        let now = Instant::now();

        while now.elapsed() < interval {
            if let Some(command) = get_command(interval - now.elapsed()) {
                match command {
                    Command::Quit => {
                        done = true;
                        break;
                    }
                    Command::Turn(towards) => {
                        if direction != towards && opposite_direction(direction) != towards {
                            snake::set_direction(&mut game.snake, towards);
                        }
                    }
                }
            }
        }

        if has_collided_with_wall(game) || has_bitten_itself(game) {
            done = true;
        } else {
            snake::slither(&mut game.snake);

            if let Some(food_point) = game.food {
                if snake::get_head_point(&game.snake) == food_point {
                    snake::grow(&mut game.snake);
                    place_food(game);
                    game.score += 1;

                    if game.score % ((game.width * game.height) / MAX_SPEED) == 0 {
                        game.speed += 1;
                    }
                }
            }

            render(game);
        }
    }

    restore_ui(game);

    println!("Game Over! Your score is {}", game.score);
}

fn restore_ui(game: &mut Game) {
    let (cols, rows) = game.original_terminal_size;
    game.stdout
        .execute(SetSize(cols, rows))
        .unwrap()
        .execute(Clear(ClearType::All))
        .unwrap()
        .execute(Show)
        .unwrap()
        .execute(ResetColor)
        .unwrap();
    disable_raw_mode().unwrap();
}

pub fn place_food(game: &mut Game) {
    loop {
        let random_x = rand::thread_rng().gen_range(0, game.width);
        let random_y = rand::thread_rng().gen_range(0, game.height);

        let i_x = i16::try_from(random_x).expect("Can't have a negative x coordinate!");
        let i_y = i16::try_from(random_y).expect("Can't have a negative y coordinate!");

        let vector = Vector::new(i_x, i_y);

        if !snake::contains_point(&game.snake, &vector) {
            game.food = Some(vector);
            break;
        }
    }
}

pub fn prepare_ui(game: &mut Game) {
    enable_raw_mode().unwrap();
    game.stdout
        .execute(SetSize(game.width + 3, game.height + 3))
        .unwrap()
        .execute(Clear(ClearType::All))
        .unwrap()
        .execute(Hide)
        .unwrap();
}

pub fn calculate_interval(game: &mut Game) -> Duration {
    let speed = MAX_SPEED - game.speed;

    Duration::from_millis(
        (MIN_INTERVAL + (((MAX_INTERVAL - MIN_INTERVAL) / MAX_SPEED) * speed)) as u64,
    )
}

fn get_command(wait_for: Duration) -> Option<Command> {
    let key_event = wait_for_key_event(wait_for)?;

    match key_event.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Command::Quit),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                Some(Command::Quit)
            } else {
                None
            }
        }
        KeyCode::Up => Some(Command::Turn(Direction::Up)),
        KeyCode::Right => Some(Command::Turn(Direction::Right)),
        KeyCode::Down => Some(Command::Turn(Direction::Down)),
        KeyCode::Left => Some(Command::Turn(Direction::Left)),
        _ => None,
    }
}

pub fn wait_for_key_event(wait_for: Duration) -> Option<KeyEvent> {
    if poll(wait_for).ok()? {
        let event = read().ok()?;
        if let Event::Key(key_event) = event {
            return Some(key_event);
        }
    }

    None
}

fn has_collided_with_wall(game: &mut Game) -> bool {
    let head_point = snake::get_head_point(&game.snake);

    let i_width = i16::try_from(game.width).expect("Can't have a negative width!");
    let i_height = i16::try_from(game.height).expect("Can't have a negative height!");

    match snake::get_direction(&game.snake) {
        Direction::Up => head_point.y == 0,
        Direction::Right => head_point.x == i_width - 1,
        Direction::Down => head_point.y == i_height - 1,
        Direction::Left => head_point.x == 0,
    }
}

fn has_bitten_itself(game: &mut Game) -> bool {
    let next_head_point = move_vector_to(
        &snake::get_head_point(&game.snake),
        snake::get_direction(&game.snake),
        1,
    );
    let mut next_body_points = snake::get_body_point(&game.snake).clone();
    next_body_points.remove(next_body_points.len() - 1);
    next_body_points.remove(0);

    next_body_points.contains(&next_head_point)
}

pub fn render(game: &mut Game) {
    draw_borders(game);
    draw_background(game);
    draw_food(game);
    draw_snake(game);
}

pub fn draw_snake(game: &mut Game) {
    let fg = SetForegroundColor(match game.speed % 3 {
        0 => Color::Green,
        1 => Color::Cyan,
        _ => Color::Yellow,
    });
    game.stdout.execute(fg).unwrap();

    let body_points = snake::get_body_point(&game.snake);
    for (i, body) in body_points.iter().enumerate() {
        let previous = if i == 0 { None } else { body_points.get(i - 1) };
        let next = body_points.get(i + 1);
        let symbol = if let Some(&next) = next {
            if let Some(&previous) = previous {
                if previous.x == next.x {
                    '║'
                } else if previous.y == next.y {
                    '═'
                } else {
                    let d = move_vector_to(body, Direction::Down, 1);
                    let r = move_vector_to(body, Direction::Right, 1);
                    let u = if body.y == 0 {
                        body.clone()
                    } else {
                        move_vector_to(body, Direction::Up, 1)
                    };
                    let l = if body.x == 0 {
                        body.clone()
                    } else {
                        move_vector_to(body, Direction::Left, 1)
                    };
                    if (next == d && previous == r) || (previous == d && next == r) {
                        '╔'
                    } else if (next == d && previous == l) || (previous == d && next == l) {
                        '╗'
                    } else if (next == u && previous == r) || (previous == u && next == r) {
                        '╚'
                    } else {
                        '╝'
                    }
                }
            } else {
                'O'
            }
        } else if let Some(&previous) = previous {
            if body.y == previous.y {
                '═'
            } else {
                '║'
            }
        } else {
            panic!("Invalid snake body point.");
        };

        let u_x = u16::try_from(body.x).expect("Can't have a negative x coordinate!");
        let u_y = u16::try_from(body.y).expect("Can't have a negative y coordinate!");

        game.stdout
            .execute(MoveTo(u_x + 1, u_y + 1))
            .unwrap()
            .execute(Print(symbol))
            .unwrap();
    }
}

pub fn draw_food(game: &mut Game) {
    game.stdout
        .execute(SetForegroundColor(Color::White))
        .unwrap();

    for food in game.food.iter() {
        let u_x = u16::try_from(food.x).expect("Can't have a negative x coordinate!");
        let u_y = u16::try_from(food.y).expect("Can't have a negative y coordinate!");

        game.stdout
            .execute(MoveTo(u_x + 1, u_y + 1))
            .unwrap()
            .execute(Print("•"))
            .unwrap();
    }
}

pub fn draw_background(game: &mut Game) {
    game.stdout.execute(ResetColor).unwrap();

    for y in 1..game.height + 1 {
        for x in 1..game.width + 1 {
            game.stdout
                .execute(MoveTo(x, y))
                .unwrap()
                .execute(Print(" "))
                .unwrap();
        }
    }
}

pub fn draw_borders(game: &mut Game) {
    game.stdout
        .execute(SetForegroundColor(Color::DarkGrey))
        .unwrap();

    for y in 0..game.height + 2 {
        game.stdout
            .execute(MoveTo(0, y))
            .unwrap()
            .execute(Print("#"))
            .unwrap()
            .execute(MoveTo(game.width + 1, y))
            .unwrap()
            .execute(Print("#"))
            .unwrap();
    }

    for x in 0..game.width + 2 {
        game.stdout
            .execute(MoveTo(x, 0))
            .unwrap()
            .execute(Print("#"))
            .unwrap()
            .execute(MoveTo(x, game.height + 1))
            .unwrap()
            .execute(Print("#"))
            .unwrap();
    }

    game.stdout
        .execute(MoveTo(0, 0))
        .unwrap()
        .execute(Print("#"))
        .unwrap()
        .execute(MoveTo(game.width + 1, game.height + 1))
        .unwrap()
        .execute(Print("#"))
        .unwrap()
        .execute(MoveTo(game.width + 1, 0))
        .unwrap()
        .execute(Print("#"))
        .unwrap()
        .execute(MoveTo(0, game.height + 1))
        .unwrap()
        .execute(Print("#"))
        .unwrap();
}
