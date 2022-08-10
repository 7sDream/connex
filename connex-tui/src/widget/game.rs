use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::{canvas::Canvas, Block, Borders, Widget};

use connex::{Command, Direction, World};

use super::canvas::Painter;

#[derive(Debug, Clone, Default)]
pub struct Game {
    game: connex::Game,
    edit_mode: bool,
}

impl Game {
    pub fn new(game: connex::Game) -> Self {
        Self { game, edit_mode: false }
    }

    pub fn edit_mode(&mut self, enable: bool) {
        self.edit_mode = enable;
    }

    pub fn reset(&mut self, world: World) {
        self.game.apply(Command::Reset(world));
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if self.edit_mode {
            if let KeyCode::Char(c) = key.code {
                if let Ok(block) = c.to_string().parse() {
                    self.game.apply(Command::ReplaceCursorBlock(block));
                    return;
                }
            }
        }

        let command = match key.code {
            KeyCode::Char('k' | 'w') => Command::MoveCursor(Direction::Up),
            KeyCode::Char('l' | 'd') => Command::MoveCursor(Direction::Right),
            KeyCode::Char('j' | 's') => Command::MoveCursor(Direction::Down),
            KeyCode::Char('h' | 'a') => Command::MoveCursor(Direction::Left),
            KeyCode::Char(' ') | KeyCode::Enter => Command::TurnCursorBlock,
            _ => return,
        };

        self.game.apply(command);
    }

    pub fn solved(&self) -> bool {
        self.game.solved()
    }

    pub fn into_inner(self) -> connex::Game {
        self.game
    }
}

impl Widget for &Game {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let solved = self.game.solved();
        let (row, col) = self.game.cursor();

        let canvas_painter = Painter::new(self.game.world(), &area);
        let canvas_widget = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| {
                canvas_painter.draw(
                    ctx,
                    |i, j| solved || i == row && j == col, // line highlight
                    |i, j| self.edit_mode || (!solved && i == row && j == col), // need boundary
                )
            })
            .x_bounds(canvas_painter.x_bound())
            .y_bounds(canvas_painter.y_bound());
        canvas_widget.render(area, buf);
    }
}
