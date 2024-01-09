use std::{
    io::{stdout, Write},
    process,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    style::{Color, Print, PrintStyledContent, SetBackgroundColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, SetSize},
    ExecutableCommand, QueueableCommand,
};

const TICK_RATE: u64 = 1000 / 120;

type Result<T> = std::result::Result<T, std::io::Error>;

struct Snake {
    x: u16,
    y: u16,
    max_x: u16,
    max_y: u16,
}

impl Snake {
    fn new(max_x: u16, max_y: u16) -> Self {
        Self {
            x: 0,
            y: 0,
            max_x,
            max_y,
        }
    }

    fn write_and_move<T: Write + ?Sized>(&mut self, output: &mut T) -> Result<()> {
        output
            .queue(MoveTo(self.x, self.y))?
            .queue(PrintStyledContent("█".white()))?;

        self.x += 1;
        if self.x >= self.max_x {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.max_y {
            self.y = 0;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let (mut x, mut y) = size()?;
    let mut snake = Snake::new(x, y);
    let mut stdout = stdout();
    enable_raw_mode()?;

    stdout.execute(SetSize(x, y))?.execute(Hide)?.flush()?;

    loop {
        if poll(Duration::from_millis(TICK_RATE))? {
            match read()? {
                Event::Key(event) => {
                    if let KeyCode::Char('c') = event.code {
                        if let KeyModifiers::CONTROL = event.modifiers {
                            stdout.queue(Print("Pressed"))?;
                            disable_raw_mode()?;
                            process::exit(0);
                        }
                    }
                }
                Event::Resize(new_x, new_y) => {
                    x = new_x;
                    y = new_y;
                    snake.max_x = x;
                    snake.max_y = y;
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
    }
}
