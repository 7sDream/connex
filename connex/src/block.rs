use core::{
    fmt::{Display, Write},
    str::FromStr,
};

/// Direction.
///
/// It has different meaning when placed in different variant of [`Block`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    /// Create a random direction.
    #[cfg(feature = "random")]
    pub fn random<R: rand::Rng>(mut r: R) -> Self {
        match r.gen_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            _ => Direction::Left,
        }
    }

    /// Get result of turn clockwise.
    pub fn turn(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    /// Check if is in horizontal direction.
    pub fn horizontal(&self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Check if in vertical direction.
    pub fn vertical(&self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    /// Get opposite direction.
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

/// A rotatable block.
///
/// Each invariant has a character representation:
///
/// - for [`Block::Empty`], the character is space(` `).
/// - for [`Block::Endpoint`], the character is arrow to it's [`Direction`]: `^`, `>`, `v`, `<`.
/// - for [`Block::Through`], the character is `/` and `-`.
/// - for [`Block::Turn`], [`Block::Fork`] and [`Block::Cross`], the character is a number in the graph[^1] bellow:
///
/// ```none
///           
///  7- -8- -9
///  |   |   |
///
///  |   |   |
///  4- -5- -6
///  |   |   |
///
///  |   |   |
///  1- -2- -3
///            
/// ```
///
/// [^1]: `-`/`|` means passable direction, center number is the character for that type of block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    /// Empty block.
    Empty,
    /// `Endpoint` can start/stop a link, from the facing [`Direction`].
    Endpoint(Direction),
    /// `Through` can connect two opposite directions,
    /// so [`Direction::Up`] has same meaning as [`Direction::Down`] in this variant,
    /// same for [`Direction::Left`] and [`Direction::Right`].
    Through(Direction),
    /// Turn can connect two adjacent direction.
    /// The [`Direction`] is determined by clockwise rotation, direction before rotation is stored.
    ///
    /// ## Example
    ///
    /// `|_` = from up(12 o'clock) to right(3 o'clock) is clockwise rotation,
    /// so direction of this turn is [`Direction::Up`].
    Turn(Direction),
    /// Fork can connect three direction.
    /// The stored [`Direction`] is the direction that **can't** be connected.
    Fork(Direction),
    /// Cross is a four way junction.
    Cross,
}

impl FromStr for Block {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            " " => Ok(Self::Empty),
            "^" => Ok(Self::Endpoint(Direction::Up)),
            ">" => Ok(Self::Endpoint(Direction::Right)),
            "v" => Ok(Self::Endpoint(Direction::Down)),
            "<" => Ok(Self::Endpoint(Direction::Left)),
            "/" => Ok(Self::Through(Direction::Up)),
            "-" => Ok(Self::Through(Direction::Left)),
            "1" => Ok(Self::Turn(Direction::Up)),
            "7" => Ok(Self::Turn(Direction::Right)),
            "9" => Ok(Self::Turn(Direction::Down)),
            "3" => Ok(Self::Turn(Direction::Left)),
            "8" => Ok(Self::Fork(Direction::Up)),
            "6" => Ok(Self::Fork(Direction::Right)),
            "2" => Ok(Self::Fork(Direction::Down)),
            "4" => Ok(Self::Fork(Direction::Left)),
            "5" => Ok(Self::Cross),
            _ => Err(()),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(match self {
            Self::Empty => ' ',
            Self::Endpoint(Direction::Up) => '^',
            Self::Endpoint(Direction::Right) => '^',
            Self::Endpoint(Direction::Down) => 'v',
            Self::Endpoint(Direction::Left) => '<',
            Self::Through(Direction::Up | Direction::Down) => '/',
            Self::Through(Direction::Left | Direction::Right) => '-',
            Self::Turn(Direction::Up) => '1',
            Self::Turn(Direction::Right) => '7',
            Self::Turn(Direction::Down) => '9',
            Self::Turn(Direction::Left) => '3',
            Self::Fork(Direction::Up) => '8',
            Self::Fork(Direction::Right) => '6',
            Self::Fork(Direction::Down) => '2',
            Self::Fork(Direction::Left) => '4',
            Self::Cross => '5',
        })
    }
}

impl Block {
    /// Create a random block.
    #[cfg(feature = "random")]
    pub fn random<R: rand::Rng>(mut r: R, allow_empty: bool) -> Self {
        let ty_start = if allow_empty { 0 } else { 1 };
        let ty = r.gen_range(ty_start..=5);

        match ty {
            0 => Self::Empty,
            1 => Self::Endpoint(Direction::random(r)),
            2 => Self::Through(Direction::random(r)),
            3 => Self::Turn(Direction::random(r)),
            4 => Self::Fork(Direction::random(r)),
            _ => Self::Cross,
        }
    }

    /// Shuffle self, make direction random.
    #[cfg(feature = "random")]
    pub fn shuffle<R: rand::Rng>(&mut self, mut r: R) {
        match self {
            Self::Endpoint(s) => *s = Direction::random(&mut r),
            Self::Through(s) => *s = Direction::random(&mut r),
            Self::Turn(s) => *s = Direction::random(&mut r),
            Self::Fork(s) => *s = Direction::random(&mut r),
            _ => (),
        }
    }

    /// Get result of turn this block clockwise.
    pub fn turn(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Endpoint(t) => Self::Endpoint(t.turn()),
            Self::Through(t) => Self::Through(t.turn()),
            Self::Turn(t) => Self::Turn(t.turn()),
            Self::Fork(t) => Self::Fork(t.turn()),
            Self::Cross => Self::Cross,
        }
    }

    /// Turn this block clockwise.
    pub fn turn_me(&mut self) {
        if let Some(t) = self.direction_mut() {
            *t = t.turn()
        }
    }

    /// Check if this block is passable to a direction.
    pub fn passable(&self, rhs: Direction) -> bool {
        match self {
            Self::Empty => false,
            Self::Endpoint(t) => t == &rhs,
            Self::Through(t) => t.horizontal() == rhs.horizontal(),
            Self::Turn(t) => t == &rhs || t.turn() == rhs,
            Self::Fork(t) => t != &rhs,
            Self::Cross => true,
        }
    }

    /// Get direction.
    pub fn direction(&self) -> Option<Direction> {
        match self {
            Self::Endpoint(t) | Self::Through(t) | Self::Turn(t) | Self::Fork(t) => Some(*t),
            _ => None,
        }
    }

    /// Get direction, mutable.
    pub fn direction_mut(&mut self) -> Option<&mut Direction> {
        match self {
            Self::Endpoint(t) | Self::Through(t) | Self::Turn(t) | Self::Fork(t) => Some(t),
            _ => None,
        }
    }

    /// Check if this block is fit another block at given side.
    pub fn fit(&self, side: Direction, other: &Self) -> bool {
        self.passable(side) == other.passable(side.opposite())
    }
}
