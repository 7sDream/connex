use connex::World;
use crossterm::event::KeyCode;

use crate::{app::App, world::WorldWidget};

#[derive(Debug, Clone)]
pub struct Editor {
    world: WorldWidget,
}

impl Editor {
    pub fn new(height: usize, width: usize) -> Self {
        let mut world = WorldWidget::default();
        world.reset(World::empty(height, width));
        world.edit_mode(true);
        Self { world }
    }
}

impl App for Editor {
    type Output = String;

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.world.on_key(key);

        !matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
    }

    fn on_tick(&mut self) {}

    fn draw<B: tui::backend::Backend>(&self, f: &mut tui::Frame<B>) {
        f.render_widget(&self.world, f.size())
    }

    fn output(self) -> Self::Output {
        format!("{}", self.world.into_inner())
    }
}
