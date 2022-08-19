use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent};
use once_cell::sync::Lazy;
use rand::thread_rng;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block as TuiBlock, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_markup::generator::TuiTextGenerator;

use connex::World;
use connex_levels::LEVELS;

static HELP_TEXT: Lazy<Text<'static>> = Lazy::new(compile_help_text);

use crate::{app::App, widget::Game as GameWidget};

fn compile_help_text() -> Text<'static> {
    let gen = TuiTextGenerator::new(|tag: &str| {
        Some(match tag {
            "h1" => Style::default()
                .bg(Color::White)
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            "h2" => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            "goal" => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            "action" => Style::default().fg(Color::Cyan),
            "kbd" => Style::default().fg(Color::Green),
            _ => return None,
        })
    });
    tui_markup::compile_with(include_str!("game_help.txt"), gen).unwrap()
}

enum Page {
    Gaming,
    Help,
}

pub struct Game {
    page: Page,
    level: Option<usize>,
    game_widget: GameWidget,
}

impl Default for Game {
    fn default() -> Self {
        let mut state = Game {
            page: Page::Gaming,
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

impl Game {
    fn on_key_common(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('?') => match self.page {
                Page::Gaming => self.page = Page::Help,
                Page::Help => self.page = Page::Gaming,
            },
            KeyCode::Char('q') | KeyCode::Esc => return false,
            _ => (),
        };

        true
    }

    fn on_key_gaming(&mut self, key: KeyEvent) -> bool {
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
            _ => (),
        }

        true
    }

    fn on_key_help(&mut self, _key: KeyEvent) -> bool {
        true
    }

    fn draw_gaming<B: Backend>(&self, f: &mut Frame<B>) {
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
        let status_bar_widget = Paragraph::new("Press ? to see help page")
            .alignment(Alignment::Center)
            .block(TuiBlock::default().borders(Borders::ALL));
        f.render_widget(status_bar_widget, status_bar_rect);
    }

    fn draw_help<B: Backend>(&self, f: &mut Frame<B>) {
        let p = Paragraph::new(HELP_TEXT.clone())
            .block(TuiBlock::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);
        f.render_widget(p, f.size());
    }
}

impl App for Game {
    type Output = ();

    fn on_key(&mut self, key: KeyEvent) -> bool {
        if !self.on_key_common(key) {
            return false;
        }

        match self.page {
            Page::Gaming => self.on_key_gaming(key),
            Page::Help => self.on_key_help(key),
        }
    }

    fn on_tick(&mut self) {}

    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        match self.page {
            Page::Gaming => self.draw_gaming(f),
            Page::Help => self.draw_help(f),
        }
    }

    fn output(self) -> Self::Output {}
}
