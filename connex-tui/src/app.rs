use std::{
    borrow::Cow,
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
    widgets::{canvas::Canvas, Block as TuiBlock, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use connex_levels::LEVELS;

use connex::World;

const TICK_RATE: Duration = std::time::Duration::from_millis(20);

pub struct State {
    level: Option<usize>,
    solved: bool,
    col: usize,
    row: usize,
    world: World,
}

impl State {
    pub fn new() -> Self {
        let mut state = State {
            level: None,
            solved: false,
            col: 0,
            row: 0,
            world: World::empty(1, 1),
        };

        if !LEVELS.is_empty() {
            state.start_level(0);
        }

        state
    }

    fn start_level(&mut self, level: usize) {
        assert!(level < LEVELS.len());

        self.solved = false;
        self.col = 0;
        self.row = 0;

        self.world = connex_levels::LEVELS[level].parse().unwrap();
        self.world.shuffle(thread_rng());

        self.level.replace(level);
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        if let Some(level) = self.level {
            if !self.solved {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return false,
                    KeyCode::Char('k' | 'w') | KeyCode::Up if self.row > 0 => self.row -= 1,
                    KeyCode::Char('l' | 'd') | KeyCode::Right if self.col < self.world.width() - 1 => self.col += 1,
                    KeyCode::Char('j' | 's') | KeyCode::Down if self.row < self.world.height() - 1 => self.row += 1,
                    KeyCode::Char('h' | 'a') | KeyCode::Left if self.col > 0 => self.col -= 1,
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        self.world.get_mut(self.row, self.col).unwrap().turn_me();
                        self.solved = self.world.check();
                    }
                    _ => (),
                };
            }

            if let KeyCode::Char('r') = key.code {
                self.start_level(level);
            };
        }

        match key.code {
            KeyCode::Char(']') if !LEVELS.is_empty() => {
                self.start_level((self.level.map(|x| x + 1).unwrap_or_default()) % LEVELS.len())
            }
            KeyCode::Char('[') if self.level.is_some() => {
                self.start_level((self.level.map(|x| x + LEVELS.len() - 1)).unwrap_or_default() % LEVELS.len())
            }
            KeyCode::Char('q') | KeyCode::Esc => return false,
            _ => (),
        }

        true
    }

    pub fn on_tick(&mut self) {}

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.size());

        let title_rect = chunks[0];
        let mut title_color = Style::default();
        if self.solved {
            title_color = title_color.fg(Color::Green);
        }
        let title = if let Some(level) = self.level {
            Cow::Owned(format!("Connex TUI - Level {level:03}"))
        } else {
            Cow::Borrowed("Connex TUI")
        };
        let title_widget = Paragraph::new(Span::styled(title, title_color))
            .alignment(Alignment::Center)
            .block(TuiBlock::default().borders(Borders::ALL));
        f.render_widget(title_widget, title_rect);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(chunks[1]);

        let level_rect = main_chunks[0];
        let mut level_list: Vec<_> = (0..LEVELS.len())
            .map(|n| format!("{n:03}"))
            .map(ListItem::new)
            .collect();
        if let Some(level) = self.level {
            let selected = level_list.get_mut(level).unwrap();
            *selected = selected.clone().style(Style::default().fg(Color::Green));
        }
        let level_widget = List::new(level_list)
            .block(TuiBlock::default().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Green));
        f.render_widget(level_widget, level_rect);

        let canvas_rect = main_chunks[1];
        if self.level.is_some() && canvas_rect.area() > 0 {
            let canvas_painter = crate::canvas::Painter::new(&self.world, &canvas_rect);
            let canvas_widget = Canvas::default()
                .block(TuiBlock::default().borders(Borders::NONE))
                .paint(|ctx| {
                    canvas_painter.draw(
                        ctx,
                        |i, j| self.solved || i == self.row && j == self.col, // line highlight
                        |i, j| !self.solved && i == self.row && j == self.col, // need boundary
                    )
                })
                .x_bounds(canvas_painter.x_bound())
                .y_bounds(canvas_painter.y_bound());
            f.render_widget(canvas_widget, canvas_rect);
        }

        let status_bar_rect = chunks[2];
        let status_bar_widget = Paragraph::new(
            "Up/Down/Left/Right/W/A/S/D to move | Space/Enter to turn block | R to restart Level | [ and ] to switch level",
        )
        .alignment(Alignment::Center)
        .block(TuiBlock::default().borders(Borders::ALL));
        f.render_widget(status_bar_widget, status_bar_rect);
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
