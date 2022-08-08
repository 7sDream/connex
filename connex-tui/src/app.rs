use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode, KeyEvent};
use rand::thread_rng;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{canvas::Canvas, Block as TuiBlock, Borders, Paragraph},
    Frame, Terminal,
};

use connex::World;

const TICK_RATE: Duration = std::time::Duration::from_millis(20);

pub struct State {
    col: usize,
    row: usize,
    world: World,
}

impl State {
    pub fn new() -> Self {
        let mut r = thread_rng();

        let mut world: World = connex_levels::LEVELS[0].parse().unwrap();

        world.shuffle(&mut r);

        State { col: 0, row: 0, world }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Char('k' | 'w') | KeyCode::Up if self.row > 0 => self.row -= 1,
            KeyCode::Char('l' | 'd') | KeyCode::Right if self.col < self.world.width() - 1 => self.col += 1,
            KeyCode::Char('j' | 's') | KeyCode::Down if self.row < self.world.height() - 1 => self.row += 1,
            KeyCode::Char('h' | 'a') | KeyCode::Left if self.col > 0 => self.col -= 1,
            KeyCode::Char(' ') | KeyCode::Enter => self.world.get_mut(self.row, self.col).unwrap().turn_me(),
            _ => (),
        };

        true
    }

    pub fn on_tick(&mut self) {}

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        let solved = self.world.check();
        let mut title_color = Style::default();
        if solved {
            title_color = title_color.fg(Color::Green);
        }
        let title = Paragraph::new(Span::styled("Connex TUI", title_color))
            .alignment(Alignment::Center)
            .block(TuiBlock::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let canvas_painter = crate::canvas::Painter::new(&self.world, &chunks[1]);
        let canvas = Canvas::default()
            .block(TuiBlock::default().borders(Borders::NONE))
            .paint(|ctx| canvas_painter.draw(ctx, |i, j| solved || i == self.row && j == self.col))
            .x_bounds(canvas_painter.x_bound())
            .y_bounds(canvas_painter.y_bound());
        f.render_widget(canvas, chunks[1]);
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let mut state = State::new();

    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| state.draw(f))?;

        let timeout = TICK_RATE.checked_sub(last_tick.elapsed()).unwrap_or(Duration::ZERO);
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if !state.on_key(key) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            state.on_tick();
            last_tick = Instant::now();
        }
    }

    Ok(())
}
