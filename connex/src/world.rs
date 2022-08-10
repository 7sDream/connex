use alloc::{format, string::String, vec::Vec};
use core::{
    fmt::{Debug, Display, Write},
    str::FromStr,
};

use crate::{Block, Towards};

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
    width: usize,
    height: usize,
    blocks: Vec<Block>,
}

impl Default for World {
    fn default() -> Self {
        World::empty(1, 1)
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
            .parse::<usize>()
            .map_err(|e| format!("{}", e))?;
        let width = hw
            .next()
            .ok_or("can't get width of world")?
            .parse::<usize>()
            .map_err(|e| format!("{}", e))?;

        if height == 0 {
            return Err("world height must not zero".into());
        }

        if width == 0 {
            return Err("world width must not zero".into());
        }

        let size = height.checked_mul(width).ok_or("too many blocks")?;

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

        if blocks.len() != size {
            return Err("blocks count not match".into());
        }

        Ok(Self { width, height, blocks })
    }
}

impl Display for World {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{},{}\n", self.height, self.width))?;
        for row in 0..self.height {
            for col in 0..self.width {
                Display::fmt(self.get(row, col).unwrap(), f)?;
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl World {
    /// Create a all empty world in given size.
    pub fn empty(height: usize, width: usize) -> Self {
        Self::new_with(height, width, |_, _| Block::Empty)
    }

    /// Create a given size world using an init function, this function will be called in each block,
    /// given argument of `row` and `col`, start from zero.
    pub fn new_with<F>(height: usize, width: usize, mut f: F) -> Self
    where
        F: FnMut(usize, usize) -> Block,
    {
        let size = height.checked_mul(width);
        assert!(size.is_some(), "too many blocks");
        let size = size.unwrap();

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
    pub fn new_from_blocks(height: usize, width: usize, blocks: Vec<Block>) -> Self {
        assert!(height > 0, "world height must not zero");
        assert!(width > 0, "world width must not zero");

        let size = height.checked_mul(width);
        assert!(size.is_some(), "too many blocks");
        let size = size.unwrap();

        assert!(size == blocks.len());

        Self { height, width, blocks }
    }

    /// Shuffle all blocks.
    #[cfg(feature = "random")]
    pub fn shuffle<R: rand::Rng>(&mut self, mut r: R) {
        for block in &mut self.blocks {
            block.shuffle(&mut r);
        }
    }

    /// Get height of world.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get width of world.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get a block in given location, return None if out of range.
    pub fn get(&self, row: usize, col: usize) -> Option<&Block> {
        self.blocks.get(row * self.width + col)
    }

    /// get a mutable block in given location, return None if out of range.
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Block> {
        self.blocks.get_mut(row * self.width + col)
    }

    /// Get inner blocks.
    pub fn into_inner(self) -> Vec<Block> {
        self.blocks
    }

    /// Turn a block in given location.
    ///
    /// ## Panics
    ///
    /// If location out of range.
    pub fn turn(&mut self, row: usize, col: usize) {
        self.get_mut(row, col).expect("block index out of range").turn_me();
    }

    fn check_cell_with_right_down(&self, row: usize, col: usize) -> bool {
        let cell = self.get(row, col).unwrap();

        if row == 0 && cell.open(Towards::Up)
            || row == self.height - 1 && cell.open(Towards::Down)
            || col == 0 && cell.open(Towards::Left)
            || col == self.width - 1 && cell.open(Towards::Right)
        {
            return false;
        }

        let next_col = col + 1;
        // right cell exists
        if next_col < self.width {
            let right = self.get(row, next_col).unwrap();
            if !cell.fit(Towards::Right, right) {
                return false;
            }
        }

        let next_row = row + 1;
        // down cell exists
        if next_row < self.height {
            let down = self.get(next_row, col).unwrap();
            if !cell.fit(Towards::Down, down) {
                return false;
            }
        }

        true
    }

    /// Check if this world's blocks is all fit.
    pub fn solved(&self) -> bool {
        (0..self.height).all(|row| (0..self.width).all(|col| self.check_cell_with_right_down(row, col)))
    }
}
