use crate::{Block, Direction, World};

/// Command is game control command.
#[derive(Debug, Clone)]
pub enum Command {
    /// Reset game world, use to switch level or restart level.
    Reset(World),
    /// Move cursor one block towards given direction.
    MoveCursor(Direction),
    /// Turn block under cursor clockwise.
    TurnCursorBlock,
    /// Turn block at given index clockwise.
    TurnBlock(usize, usize),
    /// Rotate whole world, with or without block turn.
    RotateWholeWorld(bool),
    /// Replace current block.
    ReplaceCursorBlock(Block),
    /// Replace block at given index.
    ReplaceBlock(usize, usize, Block),
    /// Insert a row of empty block at given index.
    InsertRow(usize),
    /// Insert a column of empty block  at given index.
    InsertColumn(usize),
    /// Remove a row at given index.
    RemoveRow(usize),
    /// Remove a row at given index.
    RemoveColumn(usize),
}

/// Game accept standard commands to a game world, make it playable.
#[derive(Debug, Clone)]
pub struct Game {
    world: World,
    row: usize,
    col: usize,
    solved: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(World::default())
    }
}

impl Game {
    /// Create a new game.
    pub fn new(world: World) -> Self {
        Self {
            solved: world.solved(),
            col: 0,
            row: 0,
            world,
        }
    }

    /// Get cursor.
    pub fn cursor(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    /// Check if current game world is in solved state.
    pub fn solved(&self) -> bool {
        self.solved
    }

    /// Get inner game world reference.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get inner game world.
    pub fn into_inner(self) -> World {
        self.world
    }

    fn reset(&mut self, world: World) {
        self.col = 0;
        self.row = 0;
        self.world = world;
        self.solved = self.world.solved();
    }

    fn move_cursor(&mut self, dir: Direction) {
        match dir {
            Direction::Up => {
                if self.row > 0 {
                    self.row -= 1
                }
            }
            Direction::Right => {
                if self.col < self.world.width() - 1 {
                    self.col += 1
                }
            }
            Direction::Down => {
                if self.row < self.world.height() - 1 {
                    self.row += 1
                }
            }
            Direction::Left => {
                if self.col > 0 {
                    self.col -= 1
                }
            }
        };
    }

    fn turn_block(&mut self, row: usize, col: usize) {
        self.world.get_mut(row, col).unwrap().turn_me();
        self.solved = self.world.solved();
    }

    fn replace_block(&mut self, row: usize, col: usize, block: Block) {
        *self.world.get_mut(row, col).unwrap() = block;
        self.solved = self.world.solved();
    }

    /// Apply a command in this game.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Reset(world) => self.reset(world),
            Command::MoveCursor(dir) => self.move_cursor(dir),
            Command::TurnCursorBlock => self.turn_block(self.row, self.col),
            Command::TurnBlock(row, col) => self.turn_block(row, col),
            Command::RotateWholeWorld(_) => unimplemented!(),
            Command::ReplaceCursorBlock(block) => self.replace_block(self.row, self.col, block),
            Command::ReplaceBlock(row, col, block) => self.replace_block(row, col, block),
            Command::InsertRow(_) => unimplemented!(),
            Command::InsertColumn(_) => unimplemented!(),
            Command::RemoveRow(_) => unimplemented!(),
            Command::RemoveColumn(_) => unimplemented!(),
        }
    }
}
