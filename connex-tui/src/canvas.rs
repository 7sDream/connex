use tui::{
    layout::Rect,
    style::Color,
    widgets::canvas::{Context, Line},
};

use connex::{Block, Side, World};

#[derive(Default, Debug, Clone)]
struct LayoutInfo {
    pub x_bound: u64,
    pub y_bound: u64,
    pub x_offset: u64,
    pub y_offset: u64,
    pub point_size: u64,
    pub block_size: u64,
}

fn gcd(a: u64, b: u64) -> u64 {
    let remainder = a % b;
    if remainder == 0 {
        b
    } else {
        gcd(b, remainder)
    }
}

fn lcm(a: u64, b: u64) -> u64 {
    a * b / gcd(a, b)
}

fn layout(rect: &Rect, canvas: &World) -> LayoutInfo {
    let rect_w = rect.width as u64 * 2;
    let rect_h = rect.height as u64 * 4;

    let radio_w = rect_w as f64 / canvas.width() as f64;
    let radio_h = rect_h as f64 / canvas.height() as f64;

    let mut info = LayoutInfo::default();

    info.point_size = lcm(rect_w, rect_h);
    info.block_size = 4 * info.point_size;

    if radio_w > radio_h {
        info.y_bound = canvas.height() as u64 * info.block_size + info.point_size;
        info.x_bound = info.y_bound * rect_w / rect_h;
        info.y_offset = 0;
        info.x_offset = (info.x_bound - canvas.width() as u64 * info.block_size) / 2;
    } else {
        info.x_bound = canvas.width() as u64 * info.block_size + info.point_size;
        info.y_bound = info.x_bound * rect_h / rect_w;
        info.x_offset = 0;
        info.y_offset = (info.y_bound - canvas.height() as u64 * info.block_size) / 2;
    }

    info
}

type BlockLine = ((u8, u8), (u8, u8));

const BL_UP: BlockLine = ((0, 2), (1, 2));
const BL_RIGHT: BlockLine = ((2, 3), (2, 4));
const BL_DOWN: BlockLine = ((3, 2), (4, 2));
const BL_LEFT: BlockLine = ((2, 0), (2, 1));
const BL_LEFT_UP: BlockLine = ((1, 2), (2, 1));
const BL_RIGHT_UP: BlockLine = ((2, 3), (1, 2));
const BL_RIGHT_DOWN: BlockLine = ((3, 2), (2, 3));
const BL_LEFT_DOWN: BlockLine = ((2, 1), (3, 2));

const BL_CENTER_ALL: &[BlockLine] = &[BL_LEFT_UP, BL_RIGHT_UP, BL_RIGHT_DOWN, BL_LEFT_DOWN];
const BL_AROUND_ALL: &[BlockLine] = &[BL_UP, BL_RIGHT, BL_DOWN, BL_LEFT];

const BL_UP_DOWN: BlockLine = ((0, 2), (4, 2));
const BL_LEFT_RIGHT: BlockLine = ((2, 0), (2, 4));

const BL_LEFT_UP_ARC: &[BlockLine] = &[BL_LEFT, BL_UP, BL_LEFT_UP];
const BL_RIGHT_UP_ARC: &[BlockLine] = &[BL_RIGHT, BL_UP, BL_RIGHT_UP];
const BL_RIGHT_DOWN_ARC: &[BlockLine] = &[BL_RIGHT, BL_DOWN, BL_RIGHT_DOWN];
const BL_LEFT_DOWN_ARC: &[BlockLine] = &[BL_LEFT, BL_DOWN, BL_LEFT_DOWN];

const BL_UP_FORK: &[BlockLine] = &[BL_RIGHT, BL_RIGHT_DOWN, BL_DOWN, BL_LEFT_DOWN, BL_LEFT];
const BL_RIGHT_FORK: &[BlockLine] = &[BL_UP, BL_LEFT_UP, BL_LEFT, BL_LEFT_DOWN, BL_DOWN];
const BL_DOWN_FORK: &[BlockLine] = &[BL_LEFT, BL_LEFT_UP, BL_UP, BL_RIGHT_UP, BL_RIGHT];
const BL_LEFT_FORK: &[BlockLine] = &[BL_UP, BL_RIGHT_UP, BL_RIGHT, BL_RIGHT_DOWN, BL_DOWN];

fn common_lines(block: &Block) -> &[&[BlockLine]] {
    match block {
        Block::Endpoint(_) => &[BL_CENTER_ALL],
        Block::Cross => &[BL_CENTER_ALL, BL_AROUND_ALL],
        _ => &[],
    }
}

fn side_lines(block: &Block) -> &[BlockLine] {
    match block {
        Block::Empty => &[],
        Block::Endpoint(s) => match s {
            Side::Up => &[BL_UP],
            Side::Right => &[BL_RIGHT],
            Side::Down => &[BL_DOWN],
            Side::Left => &[BL_LEFT],
        },
        Block::Through(Side::Up | Side::Down) => &[BL_UP_DOWN],
        Block::Through(Side::Left | Side::Right) => &[BL_LEFT_RIGHT],
        Block::Turn(s) => match s {
            Side::Up => BL_RIGHT_UP_ARC,
            Side::Right => BL_RIGHT_DOWN_ARC,
            Side::Down => BL_LEFT_DOWN_ARC,
            Side::Left => BL_LEFT_UP_ARC,
        },
        Block::Fork(s) => match s {
            Side::Up => BL_UP_FORK,
            Side::Right => BL_RIGHT_FORK,
            Side::Down => BL_DOWN_FORK,
            Side::Left => BL_LEFT_FORK,
        },
        Block::Cross => &[],
    }
}

#[derive(Debug)]
struct BlockPainter<'a, 'b> {
    canvas: &'a connex::World,
    layout: &'b LayoutInfo,
}

impl<'a, 'b> BlockPainter<'a, 'b> {
    pub fn draw(&self, ctx: &mut Context, row: usize, col: usize, highlight: bool) {
        let x_offset = self.layout.x_offset + self.layout.block_size * col as u64;
        let y_offset = self.layout.y_offset + self.layout.block_size * row as u64;
        let block = self.canvas.get(row, col).unwrap();

        let lines = common_lines(block).iter().flat_map(|a| a.iter()).chain(side_lines(block).iter());

        for ((from_y, from_x), (to_y, to_x)) in lines {
            let x1 = (x_offset + *from_x as u64 * self.layout.point_size) as f64;
            let y1 = (self.layout.y_bound - y_offset - *from_y as u64 * self.layout.point_size) as f64;
            let x2 = (x_offset + *to_x as u64 * self.layout.point_size) as f64;
            let y2 = (self.layout.y_bound - y_offset - *to_y as u64 * self.layout.point_size) as f64;

            ctx.draw(&Line { x1, y1, x2, y2, color: if highlight { Color::Green } else { Color::Reset } })
        }
    }
}

#[derive(Debug)]
pub struct Painter<'a> {
    canvas: &'a connex::World,
    layout: LayoutInfo,
}

impl<'a> Painter<'a> {
    pub fn new(canvas: &'a connex::World, rect: &Rect) -> Self {
        let layout = layout(rect, canvas);
        Self { canvas, layout }
    }

    pub fn x_bound(&self) -> [f64; 2] {
        [0.0, self.layout.x_bound as f64]
    }

    pub fn y_bound(&self) -> [f64; 2] {
        [0.0, self.layout.y_bound as f64]
    }

    pub fn draw<F>(&self, ctx: &mut Context, mut f: F)
    where
        F: FnMut(usize, usize) -> bool,
    {
        let block_painter = BlockPainter { canvas: self.canvas, layout: &self.layout };

        for i in 0..self.canvas.height() {
            for j in 0..self.canvas.width() {
                block_painter.draw(ctx, i, j, f(i, j))
            }
        }
    }
}
