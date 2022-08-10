use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::{canvas::Canvas, Block, Borders, Widget};

use connex::World;

#[derive(Debug, Clone, Default)]
pub struct WorldWidget {
    solved: bool,
    col: usize,
    row: usize,
    world: World,
    edit_mode: bool,
}

impl WorldWidget {
    pub fn edit_mode(&mut self, enable: bool) {
        self.edit_mode = enable;
    }

    pub fn reset(&mut self, world: World) {
        self.col = 0;
        self.row = 0;
        self.world = world;
        self.solved = self.world.solved();
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('k' | 'w') | KeyCode::Up if self.row > 0 => self.row -= 1,
            KeyCode::Char('l' | 'd') | KeyCode::Right if self.col < self.world.width() - 1 => self.col += 1,
            KeyCode::Char('j' | 's') | KeyCode::Down if self.row < self.world.height() - 1 => self.row += 1,
            KeyCode::Char('h' | 'a') | KeyCode::Left if self.col > 0 => self.col -= 1,
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.world.get_mut(self.row, self.col).unwrap().turn_me();
                self.solved = self.world.solved();
            }
            _ => (),
        };

        if self.edit_mode {
            if let KeyCode::Char(c) = key.code {
                if let Ok(block) = c.to_string().parse() {
                    *self.world.get_mut(self.row, self.col).unwrap() = block;
                    self.solved = self.world.solved();
                }
            }
        }
    }

    pub fn transform<F>(&mut self, f: F)
    where
        F: FnOnce(World) -> World,
    {
        let mut world = World::default();
        std::mem::swap(&mut world, &mut self.world);

        self.world = f(world);
    }

    pub fn solved(&self) -> bool {
        self.solved
    }

    pub fn into_inner(self) -> World {
        self.world
    }
}

impl Widget for &WorldWidget {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let canvas_painter = crate::canvas::Painter::new(&self.world, &area);
        let canvas_widget = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                canvas_painter.draw(
                    ctx,
                    |i, j| self.solved || i == self.row && j == self.col, // line highlight
                    |i, j| (!self.solved || self.edit_mode) && i == self.row && j == self.col, // need boundary
                )
            })
            .x_bounds(canvas_painter.x_bound())
            .y_bounds(canvas_painter.y_bound());
        canvas_widget.render(area, buf);
    }
}
