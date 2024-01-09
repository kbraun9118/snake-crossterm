use std::io::{stdout, Write};

use crossterm::{style::Print, terminal::Clear, ExecutableCommand, QueueableCommand};

fn main() -> Result<(), std::io::Error> {
    let mut stdout = stdout();
    stdout
        .queue(Clear(crossterm::terminal::ClearType::All))?
        .queue(Print("Hello\n"))?
        .flush()?;

    Ok(())
}
