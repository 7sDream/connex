use alloc::{format, string::String, vec::Vec};
use core::{
    fmt::{Debug, Display, Write},
    num::NonZeroUsize,
    str::FromStr,
};

use crate::{Block, Direction};

/// World is a connex game world.
///
/// Can be treat as a rectangle area made up of a bunch of [`Block`].
///
/// It has a string representation(used in [`core::str::FromStr`] trait implementation) in following format:
///
/// ```none
/// <height>,<width>
/// <char representation of block at (0, 0)><char representation of block at (0, 1)>...
/// <char representation of block at (1, 0)><char representation of block at (1, 1)>...
/// ...
/// ...
/// ....
/// .........................<char representation of block at (height - 1, weight - 1)>
/// ```
///
/// See [`Block`] document for blocks' representation.
#[derive(Debug, Clone)]
pub struct World {
    width: NonZeroUsize,
    height: NonZeroUsize,
    blocks: Vec<Block>,
}

impl Default for World {
    fn default() -> Self {
        World::empty(1.try_into().unwrap(), 1.try_into().unwrap())
    }
}

impl FromStr for World {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let first_line = lines.next().ok_or("missing size line")?;

        let mut hw = first_line.split(',');
        let height = hw
            .next()
            .ok_or("can't get height of world")?
            .parse::<NonZeroUsize>()
            .map_err(|e| format!("{}", e))?;
        let width = hw
            .next()
            .ok_or("can't get width of world")?
            .parse::<NonZeroUsize>()
            .map_err(|e| format!("{}", e))?;

        if height.get().checked_mul(width.get()).is_none() {
            return Err("too many blocks".into());
        }

        let mut blocks = Vec::new();

        for line in lines {
            for (i, part) in line.char_indices() {
                let block = line
                    .get(i..i + part.len_utf8())
                    .unwrap()
                    .parse()
                    .map_err(|_| format!("invalid block char: {part}"))?;
                blocks.push(block);
            }
        }

        Ok(Self::new_from_blocks(height, width, blocks))
    }
}

impl Display for World {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{},{}\n", self.height, self.width))?;
        for row in 0..self.height.get() {
            for col in 0..self.width.get() {
                Display::fmt(self.get(row, col).unwrap(), f)?;
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl World {
    #[track_caller]
    fn unchecked_size(height: usize, width: usize) -> usize {
        let size = height.checked_mul(width);
        assert!(size.is_some(), "too many blocks");
        size.unwrap()
    }

    /// Create a all empty world in given size.
    ///
    /// ## Panics
    ///
    /// height * width > usize::MAX.
    pub fn empty(height: NonZeroUsize, width: NonZeroUsize) -> Self {
        Self::new_with(height, width, |_, _| Block::Empty)
    }

    /// Create a given size world using an init function, this function will be called in each block,
    /// given argument of `row` and `col`, start from zero.
    ///
    /// ## Panics
    ///
    /// height * width > usize::MAX.
    pub fn new_with<F>(height: NonZeroUsize, width: NonZeroUsize, mut f: F) -> Self
    where
        F: FnMut(usize, usize) -> Block,
    {
        let size = Self::unchecked_size(height.get(), width.get());
        let mut blocks = Vec::with_capacity(size);

        let mut cur = 0;
        blocks.resize_with(size, move || {
            let cell = f(cur / width, cur % width);
            cur += 1;
            cell
        });

        Self::new_from_blocks(height, width, blocks)
    }

    /// Create a given size world using given blocks. blocks' size must be equal to height * width.
    ///
    /// ## Panics
    ///
    /// height * width > usize::MAX.
    pub fn new_from_blocks(height: NonZeroUsize, width: NonZeroUsize, blocks: Vec<Block>) -> Self {
        let size = Self::unchecked_size(height.get(), width.get());

        assert!(size == blocks.len(), "block size not match");

        Self { height, width, blocks }
    }

    /// Shuffle all blocks.
    #[cfg(feature = "random")]
    pub fn shuffle<R: rand::Rng>(&mut self, mut r: R) {
        for block in &mut self.blocks {
            block.shuffle(&mut r);
        }
    }

    /// Get size of the world.
    pub fn size(&self) -> (NonZeroUsize, NonZeroUsize) {
        (self.height, self.width)
    }

    /// Get height of the world.
    pub fn height(&self) -> NonZeroUsize {
        self.height
    }

    /// Get width of the world.
    pub fn width(&self) -> NonZeroUsize {
        self.width
    }

    /// Get a block in given index, return None if out of range.
    pub fn get(&self, row: usize, col: usize) -> Option<&Block> {
        self.blocks.get(row * self.width.get() + col)
    }

    /// get a mutable block in given location, return None if out of range.
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Block> {
        self.blocks.get_mut(row * self.width.get() + col)
    }

    /// Get inner blocks.
    pub fn into_inner(self) -> Vec<Block> {
        self.blocks
    }

    /// Insert a row with empty blocks at index, index range [0, world.height].
    ///
    /// ## Panics
    ///
    /// Index out of range.
    pub fn insert_row(&mut self, index: usize) {
        assert!(index <= self.height.get(), "index out of range");

        let after = self.blocks.split_off(self.width.get() * index);
        self.blocks
            .extend(core::iter::repeat(Block::Empty).take(self.width.get()));
        self.blocks.extend(after);

        self.height = NonZeroUsize::new(self.height.get() + 1).unwrap();
    }

    /// Remove row at index, index range [0, world.height).
    ///
    /// ## Panics
    ///
    /// world.height == 1 or index out of range.
    pub fn remove_row(&mut self, index: usize) {
        assert!(index < self.height.get(), "index out of range");

        let start = index * self.width.get();
        self.blocks.drain(start..start + self.width.get());

        self.height = NonZeroUsize::new(self.height.get() - 1).expect("can't remove last row");
    }

    /// Insert a column with empty blocks at index, index range [0. world.width].
    ///
    /// ## Panics
    ///
    /// Index out of range.
    pub fn insert_column(&mut self, index: usize) {
        assert!(index <= self.width.get(), "index out of range");

        let mut new_blocks = Vec::with_capacity(self.height.get() * (self.width.get() + 1));

        for row in self.blocks.chunks(self.width.get()) {
            new_blocks.extend(row[..index].iter().copied());
            new_blocks.push(Block::Empty);
            new_blocks.extend(row[index..].iter().copied());
        }

        self.blocks = new_blocks;
        self.width = NonZeroUsize::new(self.width.get() + 1).unwrap();
    }

    /// Remove column at index, index range [0, world.width).
    ///
    /// ## Panics
    ///
    /// world.width == 1 or index out of range.
    pub fn remove_column(&mut self, index: usize) {
        assert!(index < self.width.get(), "index out of range");

        self.blocks = self
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(i, b)| if i % self.width == index { None } else { Some(b) })
            .copied()
            .collect();

        self.width = NonZeroUsize::new(self.width.get() - 1).expect("can't remove last row");
    }

    /// Rotate the block at given index.
    ///
    /// ## Panics
    ///
    /// If index out of range.
    pub fn rotate(&mut self, row: usize, col: usize) {
        self.get_mut(row, col).expect("block index out of range").rotate();
    }

    fn check_block_fit_with_right_down(&self, row: usize, col: usize) -> bool {
        let block = self.get(row, col).unwrap();

        if row == 0 && block.passable(Direction::Up)
            || row == self.height.get() - 1 && block.passable(Direction::Down)
            || col == 0 && block.passable(Direction::Left)
            || col == self.width.get() - 1 && block.passable(Direction::Right)
        {
            return false;
        }

        let next_col = col + 1;
        // right block exists
        if next_col < self.width.get() {
            let right = self.get(row, next_col).unwrap();
            if !block.fit(Direction::Right, right) {
                return false;
            }
        }

        let next_row = row + 1;
        // down block exists
        if next_row < self.height.get() {
            let down = self.get(next_row, col).unwrap();
            if !block.fit(Direction::Down, down) {
                return false;
            }
        }

        true
    }

    /// Check if this world's blocks is all fit.
    pub fn solved(&self) -> bool {
        (0..self.height.get())
            .all(|row| (0..self.width.get()).all(|col| self.check_block_fit_with_right_down(row, col)))
            && self.blocks.iter().any(|b| b != &Block::Empty)
    }
}
