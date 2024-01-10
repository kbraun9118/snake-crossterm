use std::{
    io::{stdout, Write},
    panic::catch_unwind,
    process, thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, MoveTo},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    style::{Color, Print, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, SetSize},
    ExecutableCommand, QueueableCommand,
};
use snake::Snake;

// const TICK_RATE: u64 = 1000 / 120;
const TICK_RATE: Duration = Duration::from_millis(1000);

type Result<T> = std::result::Result<T, std::io::Error>;

#[derive(PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

mod snake {
    use std::{
        collections::{HashMap, VecDeque},
        io::Write,
    };

    use crossterm::{
        cursor,
        style::{self, Stylize},
        QueueableCommand,
    };

    use crate::{Direction, Result};

    pub struct Snake {
        body: VecDeque<(u16, u32)>,
        max_x: u16,
        max_y: u32,
        directon: Direction,
    }

    impl Snake {
        pub fn new(max_x: u16, max_y: u16) -> Self {
            let max_y = max_y as u32 * 2;
            let mut body = VecDeque::new();
            body.push_front((0, 0));
            body.push_front((1, 0));
            body.push_front((2, 0));
            body.push_front((3, 0));
            Self {
                body,
                max_x,
                max_y,
                directon: Direction::Right,
            }
        }

        pub fn write_and_move<T: Write + ?Sized>(&mut self, output: &mut T) -> Result<()> {
            let squares = self.body.iter().fold(HashMap::new(), |mut acc, (x, y)| {
                acc.entry((*x, *y / 2))
                    .and_modify(|e: &mut Vec<u32>| e.push(*y))
                    .or_insert(vec![*y]);
                acc
            });

            // output
            //     .queue(cursor::MoveTo(0, 1))?
            //     .queue(style::Print(format!("map: {squares:?}")))?;

            for ((x, y), mut coords) in squares {
                let coord_y = coords.pop().unwrap();
                if coords.is_empty() {
                    output
                        .queue(cursor::MoveTo(x, y as u16))?
                        .queue(style::PrintStyledContent(
                            if coord_y % 2 == 0 { "▀" } else { "▄" }.green(),
                        ))?;
                } else {
                    output
                        .queue(cursor::MoveTo(x, y as u16))?
                        .queue(style::PrintStyledContent("█".green()))?;
                }
            }

            // safe to unwrap since body will always have elements inside it
            let (mut x, mut y) = self.body.front().unwrap();

            match self.directon {
                Direction::Left => {
                    if x == 0 {
                        x = self.max_x - 1
                    } else {
                        x -= 1
                    }
                }
                Direction::Right => x += 1,
                Direction::Up => {
                    if y == 0 {
                        y = self.max_y - 1
                    } else {
                        y -= 1
                    }
                }
                Direction::Down => y += 1,
            }

            if x >= self.max_x {
                x = 0;
            }
            if y >= self.max_y {
                y = 0;
            }

            self.body.pop_back();
            self.body.push_front((x, y));

            Ok(())
        }

        pub fn set_max_x(&mut self, max_x: u16) {
            self.max_x = max_x;
        }

        pub fn set_max_y(&mut self, max_y: u16) {
            self.max_y = max_y as u32 * 2;
        }

        pub fn set_direction(&mut self, diretion: Direction) {
            if self.directon.opposite() != diretion {
                self.directon = diretion;
            }
        }
    }
}

fn game() -> Result<()> {
    let (mut x, mut y) = size()?;
    let mut snake = Snake::new(x, y);
    let mut stdout = stdout();
    enable_raw_mode()?;

    stdout.execute(SetSize(x, y))?.execute(Hide)?.flush()?;

    loop {
        let start = Instant::now();
        if poll(TICK_RATE)? {
            match read()? {
                Event::Key(event) => match event.code {
                    KeyCode::Char('c') => {
                        if let KeyModifiers::CONTROL = event.modifiers {
                            stdout.queue(Print("Pressed"))?;
                            disable_raw_mode()?;
                            process::exit(0);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('w') => snake.set_direction(Direction::Up),
                    KeyCode::Down | KeyCode::Char('s') => snake.set_direction(Direction::Down),
                    KeyCode::Left | KeyCode::Char('a') => snake.set_direction(Direction::Left),
                    KeyCode::Right | KeyCode::Char('d') => snake.set_direction(Direction::Right),
                    _ => (),
                },
                Event::Resize(new_x, new_y) => {
                    x = new_x;
                    y = new_y;
                    snake.set_max_x(x);
                    snake.set_max_y(y);
                }
                _ => (),
            }
        }
        stdout
            .queue(Clear(crossterm::terminal::ClearType::All))?
            .queue(MoveTo(0, 0))?
            .queue(SetBackgroundColor(Color::Rgb {
                r: 53,
                g: 53,
                b: 53,
            }))?
            .queue(Print(format!("Terminal size ({x}, {y})")))?;

        snake.write_and_move(&mut stdout)?;

        stdout.flush()?;
        let tick_duration = Instant::now() - start;

        println!("{tick_duration:?}");
        thread::sleep(TICK_RATE - tick_duration.min(TICK_RATE));
    }
}

fn main() {
    let _ = catch_unwind(|| game());

    let _ = disable_raw_mode().expect("Failed to disable raw mode");
    process::exit(1)
}
