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
    widgets::{canvas::Canvas, Block, Borders, Paragraph},
    Frame, Terminal,
};

use connex::{Block as Blk, Side, World};

const TICK_RATE: Duration = std::time::Duration::from_millis(20);

pub struct State {
    col: usize,
    row: usize,
    world: World,
}

impl State {
    pub fn new() -> Self {
        let mut r = thread_rng();

        let mut world = connex::World::empty(6, 4);

        *world.get_mut(0, 0).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(0, 1).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(0, 2).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(0, 3).unwrap() = Blk::Turn(Side::random(&mut r));

        *world.get_mut(1, 0).unwrap() = Blk::Through(Side::random(&mut r));
        *world.get_mut(1, 1).unwrap() = Blk::Fork(Side::random(&mut r));
        *world.get_mut(1, 2).unwrap() = Blk::Fork(Side::random(&mut r));
        *world.get_mut(1, 3).unwrap() = Blk::Endpoint(Side::random(&mut r));

        *world.get_mut(2, 0).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(2, 1).unwrap() = Blk::Fork(Side::random(&mut r));
        *world.get_mut(2, 2).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(2, 3).unwrap() = Blk::Turn(Side::random(&mut r));

        *world.get_mut(3, 0).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(3, 1).unwrap() = Blk::Cross;
        *world.get_mut(3, 2).unwrap() = Blk::Fork(Side::random(&mut r));
        *world.get_mut(3, 3).unwrap() = Blk::Fork(Side::random(&mut r));

        *world.get_mut(4, 0).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(4, 1).unwrap() = Blk::Fork(Side::random(&mut r));
        *world.get_mut(4, 2).unwrap() = Blk::Through(Side::random(&mut r));
        *world.get_mut(4, 3).unwrap() = Blk::Endpoint(Side::random(&mut r));

        *world.get_mut(5, 0).unwrap() = Blk::Endpoint(Side::random(&mut r));
        *world.get_mut(5, 1).unwrap() = Blk::Turn(Side::random(&mut r));
        *world.get_mut(5, 2).unwrap() = Blk::Endpoint(Side::random(&mut r));
        *world.get_mut(5, 3).unwrap() = Blk::Empty;

        State { col: 0, row: 0, world }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('k' | 'w') | KeyCode::Up => {
                self.row = (self.row + self.world.height() - 1) % self.world.height()
            }
            KeyCode::Char('l' | 'd') | KeyCode::Right => self.col = (self.col + 1) % self.world.width(),
            KeyCode::Char('j' | 's') | KeyCode::Down => self.row = (self.row + 1) % self.world.height(),
            KeyCode::Char('h' | 'a') | KeyCode::Left => {
                self.col = (self.col + self.world.width() - 1) % self.world.width()
            }
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
        let title = Paragraph::new(Span::styled("Connex", title_color))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let canvas_painter = crate::canvas::Painter::new(&self.world, &chunks[1]);
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
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
