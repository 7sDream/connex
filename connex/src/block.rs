use core::{
    fmt::{Display, Write},
    str::FromStr,
};

/// Towards/face to/orientation/direction.
///
/// It has different meaning when placed in different variant of [`Block`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Towards {
    /// Towards to up.
    Up,
    /// Towards to right.
    Right,
    /// Towards to down.
    Down,
    /// Towards to left.
    Left,
}

impl Towards {
    /// Create a random towards.
    #[cfg(feature = "random")]
    pub fn random<R: rand::Rng>(mut r: R) -> Self {
        match r.gen_range(0..=3) {
            0 => Towards::Up,
            1 => Towards::Right,
            2 => Towards::Down,
            _ => Towards::Left,
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

    /// Get result of opposite direction.
    pub fn inverse(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

/// An rotatable block.
///
/// Each invariant has a character representation:
///
/// - for [`Block::Empty`], the character is space(` `).
/// - for [`Block::Endpoint`], the character is arrow to it's [`Towards`]: `^`, `>`, `v`, `<`.
/// - for [`Block::Through`], the character is `/` and `-`.
/// - for [`Block::Turn`], [`Block::Fork`] and [`Block::Cross`], the character is a number in the graph[^1] bellow:
///
/// ```none
///  x   x   x
/// x7- -8- -9x
///  |   |   |
///
///  |   |   |
/// x4- -5- -6x
///  |   |   |
///
///  |   |   |
/// x1- -2- -3x
///  x   x   x
/// ```
///
/// [^1]: `-`/`|` means opened direction, `x` means closed direction, center number is the character for that type of block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    /// Empty block.
    Empty,
    /// Endpoint can start/stop a link, from the facing direction([`Towards`]).
    Endpoint(Towards),
    /// Through can connect two opposite directions,
    /// so [`Towards::Up`] == [`Towards::Down`] and [`Towards::Left`] == [`Towards::Right`]
    /// should be kept in game logic.
    Through(Towards),
    /// Turn can connect two adjacent direction.
    /// The [`Towards`] is determined by clockwise rotation, direction before rotation is stored.
    /// Example: `|_` = from up(12 o'clock) to right(3 o'clock) is clockwise rotation, so [`Towards`] is [`Towards::Up`].
    Turn(Towards),
    /// Fork can connect three direction.
    /// The [`Towards`] is the direction that **can't** be connected.
    Fork(Towards),
    /// Cross is a four way junction.
    Cross,
}

impl FromStr for Block {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            " " => Ok(Self::Empty),
            "^" => Ok(Self::Endpoint(Towards::Up)),
            ">" => Ok(Self::Endpoint(Towards::Right)),
            "v" => Ok(Self::Endpoint(Towards::Down)),
            "<" => Ok(Self::Endpoint(Towards::Left)),
            "/" => Ok(Self::Through(Towards::Up)),
            "-" => Ok(Self::Through(Towards::Left)),
            "1" => Ok(Self::Turn(Towards::Up)),
            "7" => Ok(Self::Turn(Towards::Right)),
            "9" => Ok(Self::Turn(Towards::Down)),
            "3" => Ok(Self::Turn(Towards::Left)),
            "8" => Ok(Self::Fork(Towards::Up)),
            "6" => Ok(Self::Fork(Towards::Right)),
            "2" => Ok(Self::Fork(Towards::Down)),
            "4" => Ok(Self::Fork(Towards::Left)),
            "5" => Ok(Self::Cross),
            _ => Err(()),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(match self {
            Self::Empty => ' ',
            Self::Endpoint(Towards::Up) => '^',
            Self::Endpoint(Towards::Right) => '^',
            Self::Endpoint(Towards::Down) => 'v',
            Self::Endpoint(Towards::Left) => '<',
            Self::Through(Towards::Up | Towards::Down) => '/',
            Self::Through(Towards::Left | Towards::Right) => '-',
            Self::Turn(Towards::Up) => '1',
            Self::Turn(Towards::Right) => '7',
            Self::Turn(Towards::Down) => '9',
            Self::Turn(Towards::Left) => '3',
            Self::Fork(Towards::Up) => '8',
            Self::Fork(Towards::Right) => '6',
            Self::Fork(Towards::Down) => '2',
            Self::Fork(Towards::Left) => '4',
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
            1 => Self::Endpoint(Towards::random(r)),
            2 => Self::Through(Towards::random(r)),
            3 => Self::Turn(Towards::random(r)),
            4 => Self::Fork(Towards::random(r)),
            _ => Self::Cross,
        }
    }

    /// Shuffle self, make side random.
    #[cfg(feature = "random")]
    pub fn shuffle<R: rand::Rng>(&mut self, mut r: R) {
        match self {
            Self::Endpoint(s) => *s = Towards::random(&mut r),
            Self::Through(s) => *s = Towards::random(&mut r),
            Self::Turn(s) => *s = Towards::random(&mut r),
            Self::Fork(s) => *s = Towards::random(&mut r),
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
        if let Some(t) = self.towards_mut() {
            *t = t.turn()
        }
    }

    /// Check if this block is open to a target direction.
    pub fn open(&self, rhs: Towards) -> bool {
        match self {
            Self::Empty => false,
            Self::Endpoint(t) => t == &rhs,
            Self::Through(t) => t.horizontal() == rhs.horizontal(),
            Self::Turn(t) => t == &rhs || t.turn() == rhs,
            Self::Fork(t) => t != &rhs,
            Self::Cross => true,
        }
    }

    /// Get towards direction.
    pub fn towards(&self) -> Option<Towards> {
        match self {
            Self::Endpoint(t) | Self::Through(t) | Self::Turn(t) | Self::Fork(t) => Some(*t),
            _ => None,
        }
    }

    /// Get towards direction, mutable.
    pub fn towards_mut(&mut self) -> Option<&mut Towards> {
        match self {
            Self::Endpoint(t) | Self::Through(t) | Self::Turn(t) | Self::Fork(t) => Some(t),
            _ => None,
        }
    }

    /// Check if this block is fit other block in given side.
    pub fn fit(&self, side: Towards, other: &Self) -> bool {
        self.open(side) == other.open(side.inverse())
    }
}
