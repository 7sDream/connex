use std::num::NonZeroUsize;

use connex::World;
use crossterm::event::KeyCode;

use crate::{app::App, widget::Game as GameWidget};

#[derive(Debug, Clone)]
pub struct Editor {
    game_widget: GameWidget,
}

impl Editor {
    pub fn new(height: NonZeroUsize, width: NonZeroUsize) -> Self {
        let mut game_widget = GameWidget::default();
        game_widget.reset(World::empty(height, width));
        game_widget.set_edit(true);
        Self { game_widget }
    }
}

impl App for Editor {
    type Output = String;

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.game_widget.on_key(key);

        if let KeyCode::Char('p') = key.code {
            self.game_widget.set_edit(!self.game_widget.is_edit());
        }

        !matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
    }

    fn on_tick(&mut self) {}

    fn draw<B: tui::backend::Backend>(&self, f: &mut tui::Frame<B>) {
        f.render_widget(&self.game_widget, f.size())
    }

    fn output(self) -> Self::Output {
        format!("{}", self.game_widget.into_inner().into_inner())
    }
}
