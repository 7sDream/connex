use connex::World;
use crossterm::event::KeyCode;

use crate::{app::App, widget::Game};

#[derive(Debug, Clone)]
pub struct Editor {
    game: Game,
}

impl Editor {
    pub fn new(height: usize, width: usize) -> Self {
        let mut world = Game::default();
        world.reset(World::empty(height, width));
        world.edit_mode(true);
        Self { game: world }
    }
}

impl App for Editor {
    type Output = String;

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.game.on_key(key);

        !matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
    }

    fn on_tick(&mut self) {}

    fn draw<B: tui::backend::Backend>(&self, f: &mut tui::Frame<B>) {
        f.render_widget(&self.game, f.size())
    }

    fn output(self) -> Self::Output {
        format!("{}", self.game.into_inner().into_inner())
    }
}
