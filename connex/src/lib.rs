#![warn(clippy::all)]
#![warn(missing_docs, missing_debug_implementations)]
#![deny(warnings)]
#![forbid(unsafe_code)]
#![no_std]

//! # Connex
//!
//! Basic library for connex gameplay logic.

extern crate alloc;
use alloc::vec::Vec;

/// Block sides.
/// It has different meaning when placed in different type of [`Block`]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Side {
    /// Up side
    Up,
    /// Right side
    Right,
    /// Down side
    Down,
    /// Left side
    Left,
}

impl Side {
    /// Create a random side.
    #[cfg(feature = "random")]
    pub fn random<R: rand::Rng>(mut r: R) -> Self {
        match r.gen_range(0..=3) {
            0 => Side::Up,
            1 => Side::Right,
            2 => Side::Down,
            _ => Side::Left,
        }
    }

    /// Get result of turn this side clockwise.
    pub fn turn(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    /// Check if this side is in horizontal direction.
    pub fn horizontal(&self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Check if this side is in vertical direction.
    pub fn vertical(&self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    /// Get result of inverse this side.
    pub fn inverse(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

/// An actionable block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    /// Empty block.
    Empty,
    /// Endpoint's side means output side of this block.
    Endpoint(Side),
    /// Through's side means direction of through line, so [`Side::Left`] = [`Side::Right`] and [`Side::Up`] = [`Side::Down`].
    Through(Side),
    /// Turn's side means enter side of the turn when enter -> out is clockwise
    Turn(Side),
    /// Fork's side means can't pass through which side.
    Fork(Side),
    /// Cross is a four way junction.
    Cross,
}

impl Block {
    /// Create a random block.
    #[cfg(feature = "random")]
    pub fn random<R: rand::Rng>(mut r: R, allow_empty: bool) -> Self {
        let ty_start = if allow_empty { 0 } else { 1 };
        let ty = r.gen_range(ty_start..=5);

        match ty {
            0 => Self::Empty,
            1 => Self::Endpoint(Side::random(r)),
            2 => Self::Through(Side::random(r)),
            3 => Self::Turn(Side::random(r)),
            4 => Self::Fork(Side::random(r)),
            _ => Self::Cross,
        }
    }

    /// Get result of turn this block clockwise.
    pub fn turn(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Endpoint(s) => Self::Endpoint(s.turn()),
            Self::Through(s) => Self::Endpoint(s.turn()),
            Self::Turn(s) => Self::Endpoint(s.turn()),
            Self::Fork(s) => Self::Endpoint(s.turn()),
            Self::Cross => Self::Cross,
        }
    }

    /// Turn this block clockwise.
    pub fn turn_me(&mut self) {
        match self {
            Self::Endpoint(s) => *s = s.turn(),
            Self::Through(s) => *s = s.turn(),
            Self::Turn(s) => *s = s.turn(),
            Self::Fork(s) => *s = s.turn(),
            _ => (),
        };
    }

    /// Check if this block is open to a target side.
    pub fn to(&self, side: Side) -> bool {
        match self {
            Self::Empty => false,
            Self::Endpoint(s) => *s == side,
            Self::Through(s) => s.horizontal() == side.horizontal(),
            Self::Turn(s) => *s == side || s.turn() == side,
            Self::Fork(s) => *s != side,
            Self::Cross => true,
        }
    }

    /// Check if this block is fit other block in given side.
    pub fn fit(&self, side: Side, other: &Self) -> bool {
        self.to(side) == other.to(side.inverse())
    }
}

/// World is a rectangle area made up of a bunch of blocks.
#[derive(Debug, Clone)]
pub struct World {
    width: usize,
    height: usize,
    blocks: Vec<Block>,
}

impl World {
    /// Create a all empty canvas in given size.
    pub fn empty(height: usize, width: usize) -> Self {
        Self::new_with(height, width, |_, _| Block::Empty)
    }

    /// Create a given size canvas using an init function, this function will be called in each block,
    /// given argument of `row` and `col`, start from zero.
    pub fn new_with<F>(height: usize, width: usize, mut f: F) -> Self
    where
        F: FnMut(usize, usize) -> Block,
    {
        assert!(height > 0, "canvas height must not zero");
        assert!(width > 0, "canvas width must not zero");

        let size = height * width;
        let mut cells = Vec::with_capacity(size);

        let mut cur = 0;
        cells.resize_with(size, move || {
            let cell = f(cur / width, cur % width);
            cur += 1;
            cell
        });

        Self { width, height, blocks: cells }
    }

    /// Get height of canvas.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get width of canvas.
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

        let next_col = col + 1;
        // right cell exists
        if next_col < self.width {
            let right = self.get(row, next_col).unwrap();
            if !cell.fit(Side::Right, right) {
                return false;
            }
        }

        let next_row = row + 1;
        // down cell exists
        if next_row < self.height {
            let down = self.get(next_row, col).unwrap();
            if !cell.fit(Side::Down, down) {
                return false;
            }
        }

        true
    }

    /// Check if this canvas is all fit.
    pub fn check(&self) -> bool {
        (0..self.height).all(|row| (0..self.width).all(|col| self.check_cell_with_right_down(row, col)))
    }
}
