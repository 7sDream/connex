use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent};
use once_cell::sync::Lazy;
use rand::thread_rng;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block as TuiBlock, Borders, List, ListItem, Paragraph},
    Frame,
};

use connex::World;
use connex_levels::LEVELS;

static HELP_TEXT: Lazy<Text<'static>> = Lazy::new(|| {
    tui_markup::parse("<green w>/<green a>/<green s>/<green d>: Move | <green Space>/<green Enter>: Turn | <green r>: Restart | <green [>/<green ]>: Select level").unwrap()
});

use crate::{app::App, widget::Game as GameWidget};

pub struct Game {
    level: Option<usize>,
    game_widget: GameWidget,
}

impl Default for Game {
    fn default() -> Self {
        let mut state = Game {
            level: None,
            game_widget: GameWidget::default(),
        };

        if !LEVELS.is_empty() {
            state.start_level(0);
        }

        state
    }
}

impl Game {
    fn start_level(&mut self, level: usize) {
        assert!(level < LEVELS.len());

        let mut world: World = connex_levels::LEVELS[level].parse().unwrap();
        world.shuffle(thread_rng());

        self.game_widget.reset(world);
        self.level.replace(level);
    }
}

impl App for Game {
    type Output = ();

    fn on_key(&mut self, key: KeyEvent) -> bool {
        if let Some(level) = self.level {
            if !self.game_widget.solved() {
                self.game_widget.on_key(key);
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

    fn on_tick(&mut self) {}

    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.size());

        let title_rect = chunks[0];
        let mut title_color = Style::default();
        if self.game_widget.solved() {
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
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(chunks[1]);

        let level_rect = main_chunks[0];
        let mut level_list: Vec<_> = (0..LEVELS.len())
            .map(|n| format!(" {n:03}"))
            .map(ListItem::new)
            .collect();
        if let Some(level) = self.level {
            let selected = level_list.get_mut(level).unwrap();
            *selected = selected.clone().style(Style::default().fg(Color::Green));
        }
        let level_widget = List::new(level_list)
            .block(TuiBlock::default().borders(Borders::ALL).title("Levels"))
            .highlight_style(Style::default().fg(Color::Green));
        f.render_widget(level_widget, level_rect);

        let game_widget_rect = main_chunks[1];
        if self.level.is_some() && game_widget_rect.area() > 0 {
            f.render_widget(&self.game_widget, game_widget_rect);
        }

        let status_bar_rect = chunks[2];
        let status_bar_widget = Paragraph::new(HELP_TEXT.clone())
            .alignment(Alignment::Center)
            .block(TuiBlock::default().borders(Borders::ALL));
        f.render_widget(status_bar_widget, status_bar_rect);
    }

    fn output(self) -> Self::Output {}
}
