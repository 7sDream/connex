#![warn(clippy::all)]
#![warn(missing_debug_implementations)]
#![deny(warnings)]
#![forbid(unsafe_code)]

mod app;
mod widget;

use std::{env::args, error::Error, time::Duration};

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use tui::{backend::CrosstermBackend, Terminal};

use app::App;

const TICK_RATE: Duration = std::time::Duration::from_millis(20);

fn editor_world_size() -> Option<(usize, usize)> {
    let editor_args: Vec<_> = args().skip(1).take(3).collect();
    let is_editor_mode = editor_args.get(0).map(|s| s == "editor").unwrap_or_default();

    if !is_editor_mode {
        return None;
    }

    let height = editor_args
        .get(1)
        .and_then(|h| h.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let width = editor_args
        .get(2)
        .and_then(|w| w.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);

    Some((height, width))
}

fn main() -> Result<(), Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let output = if let Some((height, width)) = editor_world_size() {
        Some(app::Editor::new(height, width).run(&mut terminal, TICK_RATE)?)
    } else {
        app::Game::default().run(&mut terminal, TICK_RATE)?;
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
