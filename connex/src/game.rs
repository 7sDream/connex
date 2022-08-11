use crate::{Block, Direction, World};

/// Command is game control command.
#[derive(Debug, Clone)]
pub enum Command {
    /// Do Nothing.
    Noop,
    /// Reset game world, use to switch level or restart level.
    Reset(World),
    /// Move cursor one block towards given direction.
    MoveCursor(Direction),
    /// Turn block under cursor clockwise.
    RotateCursorBlock,
    /// Turn block at given index clockwise.
    RotateBlock(usize, usize),
    /// Rotate whole world, with or without block rotation.
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

    /// Get col of cursor.
    pub fn col(&self) -> usize {
        self.col
    }

    /// Get row of cursor.
    pub fn row(&self) -> usize {
        self.row
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

    fn mutate_world<F>(&mut self, f: F)
    where
        F: FnOnce(&mut World),
    {
        f(&mut self.world);
        self.solved = self.world.solved();
    }

    fn reset(&mut self, mut world: World) {
        self.col = 0;
        self.row = 0;
        self.mutate_world(|old| core::mem::swap(old, &mut world));
    }

    fn move_cursor(&mut self, dir: Direction) {
        match dir {
            Direction::Up => {
                if self.row > 0 {
                    self.row -= 1
                }
            }
            Direction::Right => {
                if self.col < self.world.width().get() - 1 {
                    self.col += 1
                }
            }
            Direction::Down => {
                if self.row < self.world.height().get() - 1 {
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

    fn rotate_block(&mut self, row: usize, col: usize) {
        self.mutate_world(|w| w.get_mut(row, col).unwrap().rotate());
    }

    fn replace_block(&mut self, row: usize, col: usize, block: Block) {
        self.mutate_world(|w| *w.get_mut(row, col).unwrap() = block);
    }

    fn insert_row(&mut self, index: usize) {
        self.mutate_world(|w| w.insert_row(index));
        if self.row >= index {
            self.row += 1;
        }
    }

    fn remove_row(&mut self, index: usize) {
        if self.world.height().get() > 1 {
            self.mutate_world(|w| w.remove_row(index));
            if self.row == self.world.height().get() {
                self.row -= 1;
            }
        }
    }

    fn insert_column(&mut self, index: usize) {
        self.mutate_world(|w| w.insert_column(index));
        if self.col >= index {
            self.col += 1;
        }
    }

    fn remove_column(&mut self, index: usize) {
        if self.world.width().get() > 1 {
            self.mutate_world(|w| w.remove_column(index));
            if self.col == self.world.width().get() {
                self.col -= 1;
            }
        }
    }

    /// Apply a command in this game.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Noop => (),
            Command::Reset(world) => self.reset(world),
            Command::MoveCursor(dir) => self.move_cursor(dir),
            Command::RotateCursorBlock => self.rotate_block(self.row, self.col),
            Command::RotateBlock(row, col) => self.rotate_block(row, col),
            Command::RotateWholeWorld(_) => unimplemented!(),
            Command::ReplaceCursorBlock(block) => self.replace_block(self.row, self.col, block),
            Command::ReplaceBlock(row, col, block) => self.replace_block(row, col, block),
            Command::InsertRow(index) => self.insert_row(index),
            Command::InsertColumn(index) => self.insert_column(index),
            Command::RemoveRow(index) => self.remove_row(index),
            Command::RemoveColumn(index) => self.remove_column(index),
        }
    }
}
