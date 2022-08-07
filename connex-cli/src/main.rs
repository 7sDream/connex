// #![warn(clippy::all)]
// #![warn(missing_debug_implementations)]
// #![deny(warnings)]
#![forbid(unsafe_code)]

mod app;
mod canvas;

use std::error::Error;

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    app::run(&mut terminal)?;

    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}
