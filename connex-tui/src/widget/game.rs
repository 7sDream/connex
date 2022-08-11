use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::{canvas::Canvas, Block, Borders, Widget};

use connex::{Command, Direction, World};

use super::WorldPainter;

#[derive(Debug, Clone, Default)]
pub struct Game {
    game: connex::Game,
    edit: bool,
}

impl Game {
    pub fn new(game: connex::Game) -> Self {
        Self { game, edit: false }
    }

    pub fn is_edit(&self) -> bool {
        self.edit
    }

    pub fn set_edit(&mut self, enable: bool) {
        self.edit = enable;
    }

    pub fn reset(&mut self, world: World) {
        self.game.apply(Command::Reset(world));
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if self.edit {
            if let KeyCode::Char(c) = key.code {
                let command = match c {
                    'N' => Command::InsertRow(self.game.row() + 1),
                    'O' => Command::InsertRow(self.game.row()),
                    'D' => Command::RemoveRow(self.game.row()),
                    'A' => Command::InsertColumn(self.game.col() + 1),
                    'I' => Command::InsertColumn(self.game.col()),
                    'X' => Command::RemoveColumn(self.game.col()),
                    _ => {
                        if let Ok(block) = c.to_string().parse() {
                            Command::ReplaceCursorBlock(block)
                        } else {
                            Command::Noop
                        }
                    }
                };

                self.game.apply(command);
            }
        }

        let command = match key.code {
            KeyCode::Char('k' | 'w') => Command::MoveCursor(Direction::Up),
            KeyCode::Char('l' | 'd') => Command::MoveCursor(Direction::Right),
            KeyCode::Char('j' | 's') => Command::MoveCursor(Direction::Down),
            KeyCode::Char('h' | 'a') => Command::MoveCursor(Direction::Left),
            KeyCode::Char(' ') | KeyCode::Enter => Command::RotateCursorBlock,
            _ => Command::Noop,
        };

        self.game.apply(command);
    }

    pub fn solved(&self) -> bool {
        self.game.solved()
    }

    pub fn into_inner(self) -> connex::Game {
        self.game
    }

    fn need_highlight(&self, i: usize, j: usize) -> bool {
        // if puzzle is solved, and not in edit mode, highlight all block
        if self.solved() && !self.edit {
            return true;
        }

        // else only highlight selected block

        let (row, col) = self.game.cursor();

        i == row && j == col
    }

    fn need_boundary(&self, i: usize, j: usize) -> bool {
        // edit mode, all block need boundary to make a grid
        if self.edit {
            return true;
        }

        // normal mode, only selected block has boundary

        let (row, col) = self.game.cursor();

        i == row && j == col
    }
}

impl Widget for &Game {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let painter = WorldPainter::new(self.game.world(), &area);
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .paint(|ctx| painter.draw(ctx, |i, j| self.need_highlight(i, j), |i, j| self.need_boundary(i, j)))
            .x_bounds(painter.x_bound())
            .y_bounds(painter.y_bound());
        canvas.render(area, buf);
    }
}
