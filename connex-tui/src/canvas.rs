use tui::{
    layout::Rect,
    style::Color,
    widgets::canvas::{Context, Line},
};

use connex::{Block, Direction, World};

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
    if rect.area() == 0 {
        return LayoutInfo::default();
    }

    let rect_w = (rect.width as u64) * 2;
    let rect_h = (rect.height as u64) * 4;

    let radio_w = rect_w as f64 / canvas.width() as f64;
    let radio_h = rect_h as f64 / canvas.height() as f64;

    let mut info = LayoutInfo::default();

    info.point_size = lcm(rect_w, rect_h);
    info.block_size = 4 * info.point_size;

    if radio_w > radio_h {
        info.y_bound = canvas.height() as u64 * info.block_size + 2 * info.point_size;
        info.x_bound = info.y_bound * rect_w / rect_h;
        info.y_offset = info.point_size;
        info.x_offset = (info.x_bound - canvas.width() as u64 * info.block_size) / 2;
    } else {
        info.x_bound = canvas.width() as u64 * info.block_size + 2 * info.point_size;
        info.y_bound = info.x_bound * rect_h / rect_w;
        info.x_offset = info.point_size;
        info.y_offset = (info.y_bound - canvas.height() as u64 * info.block_size) / 2;
    }

    info
}

type BlockLine = ((u8, u8), (u8, u8));

const BL_EP_UP: BlockLine = ((0, 2), (1, 2));
const BL_EP_RIGHT: BlockLine = ((2, 3), (2, 4));
const BL_EP_DOWN: BlockLine = ((3, 2), (4, 2));
const BL_EP_LEFT: BlockLine = ((2, 0), (2, 1));
const BL_TURN_LEFT_UP: BlockLine = ((1, 2), (2, 1));
const BL_TURN_RIGHT_UP: BlockLine = ((2, 3), (1, 2));
const BL_TURN_RIGHT_DOWN: BlockLine = ((3, 2), (2, 3));
const BL_TURN_LEFT_DOWN: BlockLine = ((2, 1), (3, 2));

const BL_TURN_ALL: &[BlockLine] = &[BL_TURN_LEFT_UP, BL_TURN_RIGHT_UP, BL_TURN_RIGHT_DOWN, BL_TURN_LEFT_DOWN];
const BL_EP_ALL: &[BlockLine] = &[BL_EP_UP, BL_EP_RIGHT, BL_EP_DOWN, BL_EP_LEFT];

const BL_THROUGH_UP_DOWN: BlockLine = ((0, 2), (4, 2));
const BL_THROUGH_LEFT_RIGHT: BlockLine = ((2, 0), (2, 4));

const BL_LEFT_UP_ARC: &[BlockLine] = &[BL_EP_LEFT, BL_EP_UP, BL_TURN_LEFT_UP];
const BL_RIGHT_UP_ARC: &[BlockLine] = &[BL_EP_RIGHT, BL_EP_UP, BL_TURN_RIGHT_UP];
const BL_RIGHT_DOWN_ARC: &[BlockLine] = &[BL_EP_RIGHT, BL_EP_DOWN, BL_TURN_RIGHT_DOWN];
const BL_LEFT_DOWN_ARC: &[BlockLine] = &[BL_EP_LEFT, BL_EP_DOWN, BL_TURN_LEFT_DOWN];

const BL_UP_FORK: &[BlockLine] = &[
    BL_EP_RIGHT,
    BL_TURN_RIGHT_DOWN,
    BL_EP_DOWN,
    BL_TURN_LEFT_DOWN,
    BL_EP_LEFT,
];
const BL_RIGHT_FORK: &[BlockLine] = &[BL_EP_UP, BL_TURN_LEFT_UP, BL_EP_LEFT, BL_TURN_LEFT_DOWN, BL_EP_DOWN];
const BL_DOWN_FORK: &[BlockLine] = &[BL_EP_LEFT, BL_TURN_LEFT_UP, BL_EP_UP, BL_TURN_RIGHT_UP, BL_EP_RIGHT];
const BL_LEFT_FORK: &[BlockLine] = &[BL_EP_UP, BL_TURN_RIGHT_UP, BL_EP_RIGHT, BL_TURN_RIGHT_DOWN, BL_EP_DOWN];

const BL_BOUNDARY_UP: BlockLine = ((0, 0), (0, 4));
const BL_BOUNDARY_RIGHT: BlockLine = ((0, 4), (4, 4));
const BL_BOUNDARY_DOWN: BlockLine = ((4, 0), (4, 4));
const BL_BOUNDARY_LEFT: BlockLine = ((0, 0), (4, 0));
const BL_BOUNDARY: &[BlockLine] = &[BL_BOUNDARY_UP, BL_BOUNDARY_RIGHT, BL_BOUNDARY_DOWN, BL_BOUNDARY_LEFT];

fn common_lines(block: &Block) -> &[&[BlockLine]] {
    match block {
        Block::Endpoint(_) => &[BL_TURN_ALL],
        Block::Cross => &[BL_TURN_ALL, BL_EP_ALL],
        _ => &[],
    }
}

fn side_lines(block: &Block) -> &[BlockLine] {
    match block {
        Block::Empty => &[],
        Block::Endpoint(s) => match s {
            Direction::Up => &[BL_EP_UP],
            Direction::Right => &[BL_EP_RIGHT],
            Direction::Down => &[BL_EP_DOWN],
            Direction::Left => &[BL_EP_LEFT],
        },
        Block::Through(Direction::Up | Direction::Down) => &[BL_THROUGH_UP_DOWN],
        Block::Through(Direction::Left | Direction::Right) => &[BL_THROUGH_LEFT_RIGHT],
        Block::Turn(s) => match s {
            Direction::Up => BL_RIGHT_UP_ARC,
            Direction::Right => BL_RIGHT_DOWN_ARC,
            Direction::Down => BL_LEFT_DOWN_ARC,
            Direction::Left => BL_LEFT_UP_ARC,
        },
        Block::Fork(s) => match s {
            Direction::Up => BL_UP_FORK,
            Direction::Right => BL_RIGHT_FORK,
            Direction::Down => BL_DOWN_FORK,
            Direction::Left => BL_LEFT_FORK,
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
    fn create_line(&self, x_offset: u64, y_offset: u64, point: &BlockLine, color: Color) -> Line {
        let ((from_y, from_x), (to_y, to_x)) = point;

        let x1 = (x_offset + *from_x as u64 * self.layout.point_size) as f64;
        let y1 = (self.layout.y_bound - y_offset - *from_y as u64 * self.layout.point_size) as f64;
        let x2 = (x_offset + *to_x as u64 * self.layout.point_size) as f64;
        let y2 = (self.layout.y_bound - y_offset - *to_y as u64 * self.layout.point_size) as f64;

        Line { x1, y1, x2, y2, color }
    }

    fn draw<'i, I: IntoIterator<Item = &'i BlockLine>>(
        &self, ctx: &mut Context, row: usize, col: usize, lines: I, highlight: bool,
    ) {
        let x_offset = self.layout.x_offset + self.layout.block_size * col as u64;
        let y_offset = self.layout.y_offset + self.layout.block_size * row as u64;

        let color = if highlight { Color::Green } else { Color::Reset };

        for point in lines {
            ctx.draw(&self.create_line(x_offset, y_offset, point, color))
        }
    }

    pub fn draw_inner(&self, ctx: &mut Context, row: usize, col: usize, highlight: bool) {
        let block = self.canvas.get(row, col).unwrap();

        let lines = common_lines(block)
            .iter()
            .flat_map(|a| a.iter())
            .chain(side_lines(block).iter());

        self.draw(ctx, row, col, lines, highlight)
    }

    pub fn draw_boundary(&self, ctx: &mut Context, row: usize, col: usize, highlight: bool) {
        self.draw(ctx, row, col, BL_BOUNDARY, highlight)
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

    pub fn draw<F1, F2>(&self, ctx: &mut Context, mut highlight_pred: F1, mut boundary_pred: F2)
    where
        F1: FnMut(usize, usize) -> bool,
        F2: FnMut(usize, usize) -> bool,
    {
        let block_painter = BlockPainter {
            canvas: self.canvas,
            layout: &self.layout,
        };

        let mut boundary_position = Vec::new();

        for i in 0..self.canvas.height() {
            for j in 0..self.canvas.width() {
                block_painter.draw_inner(ctx, i, j, highlight_pred(i, j));
                if boundary_pred(i, j) {
                    boundary_position.push((i, j));
                }
            }
        }

        if !boundary_position.is_empty() {
            for (row, col) in boundary_position {
                block_painter.draw_boundary(ctx, row, col, true);
            }
        }
    }
}
