#![warn(clippy::all)]
#![warn(missing_debug_implementations)]
#![deny(warnings)]
#![forbid(unsafe_code)]

mod app;
mod canvas;
mod editor;
mod game;
mod world;

use std::{env::args, error::Error, time::Duration};

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use tui::{backend::CrosstermBackend, Terminal};

use app::App;

const TICK_RATE: Duration = std::time::Duration::from_millis(20);

fn main() -> Result<(), Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let editor = args().skip(1).take(1).next().unwrap_or_default() == "editor";

    let output = if editor {
        Some(editor::Editor::run(&mut terminal, TICK_RATE)?)
    } else {
        game::Game::run(&mut terminal, TICK_RATE)?;
        None
    };

    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;

    if let Some(output) = output {
        print!("{}", output)
    }

    Ok(())
}
