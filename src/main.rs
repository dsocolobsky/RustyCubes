use ggez::*;
use ggez::graphics::Color;
use std::time::{Duration, Instant};

const COLOR_ORANGE_DARK: Color = Color {r: 242.0/255.0, g: 125.0/255.0, b: 0.0/255.0, a: 1.0};
const COLOR_ORANGE_LIGHT: Color = Color {r: 255.0/255.0, g: 172.0/255.0, b: 84.0/255.0, a: 1.0};

const COLOR_RED_DARK: Color = Color {r: 200.0/255.0, g: 0.0/255.0, b: 0.0/255.0, a: 1.0};
const COLOR_RED_LIGHT: Color = Color {r: 250.0/255.0, g: 0.0/255.0, b: 0.0/255.0, a: 1.0};

const COLOR_WHITE: Color = ggez::graphics::WHITE;

// Here we're defining how many quickly we want our game to update. This will be
// important later so that we don't have our snake fly across the screen because
// it's moving a full tile every frame.
const UPDATES_PER_SECOND: f32 = 1.0;
// And we get the milliseconds of delay that this update rate corresponds to.
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

const BLOCK_SIZE: f32 = 32.0;
const BLOCK_INNER_SIZE: f32 = BLOCK_SIZE - 1.0;

const GRID_COLS: usize = 20;
const GRID_ROWS: usize = 10;
const GRID_SIZE: f32 = BLOCK_SIZE + 4.0;
const GRID_POS_X: f32 = 250.0;
const GRID_POS_Y: f32 = 80.0;

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 920.0;

const PIECE_ROWS: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: i16,
    y: i16
}

impl GridPosition {
    pub fn new(x: i16, y: i16) -> GridPosition {
        GridPosition {x, y}
    }
}

impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> graphics::Rect {
        graphics::Rect::new_i32(
            pos.x as i32 * GRID_SIZE as i32 + GRID_POS_X as i32 + 1,
            pos.y as i32 * GRID_SIZE as i32 + GRID_POS_Y as i32 + 1,
            GRID_SIZE as i32, GRID_SIZE as i32)
    }
}

impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

#[derive(Clone, Copy, Debug)]
enum BlockColor {
    ORANGE,
    RED,
}

impl From<BlockColor> for (Color, Color) {
    fn from(color: BlockColor) -> Self {
        match color {
            BlockColor::ORANGE => (COLOR_ORANGE_DARK, COLOR_ORANGE_LIGHT),
            BlockColor::RED => (COLOR_RED_DARK, COLOR_RED_LIGHT)
        }
    }
}

#[derive(Clone, Debug)]
struct Block {
    color: BlockColor,
    position: GridPosition,
    offset: GridPosition,
    rect: graphics::Rect,
    outer_mesh: graphics::Mesh,
    inner_mesh: graphics::Mesh,
    active: bool,
}

impl Block {
    pub fn new(ctx: &mut Context, x: i16, y: i16, color: BlockColor) -> Block {
        let position = GridPosition::from((x, y));
        let rect = graphics::Rect::from(position);

        let (dark_color, light_color) = color.into();

        let block = Block {
            color: color,
            position: position,
            rect: rect,
            offset: (0, 0).into(),
            outer_mesh: graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), 
                    rect, dark_color).unwrap(),
            inner_mesh: graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), 
                graphics::Rect::new(rect.x+2.0, rect.y+2.0, BLOCK_INNER_SIZE, BLOCK_INNER_SIZE),
                light_color).unwrap(),
            active: false,
        };

        block
    }

    pub fn empty(ctx: &mut Context, color: BlockColor) -> Block {
        Block::new(ctx, 0, 0, color)
    }

    fn update(&mut self, ctx: &mut Context, parent_position: GridPosition) {
        //self.rect.y = self.rect.y + FALLING_SPEED;
        self.position.x = parent_position.x + self.offset.x;
        self.position.y = parent_position.y + self.offset.y;

        self.rect = graphics::Rect::from(self.position);

        let (dark_color, light_color) = self.color.into();

        self.outer_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(),
            self.rect, dark_color).unwrap();

        self.inner_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(),
            self.inner_rect(), light_color).unwrap();
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        if self.active {
            graphics::draw(ctx, &self.outer_mesh, graphics::DrawParam::default()).unwrap();
            graphics::draw(ctx, &self.inner_mesh, graphics::DrawParam::default()).unwrap();
        }
        Ok(())
    }

    fn inner_rect(&self) -> graphics::Rect {
        graphics::Rect::new(self.rect.x+2.0, self.rect.y+2.0, BLOCK_INNER_SIZE, BLOCK_INNER_SIZE)
    }

    fn activate(&mut self) {
        self.active = true;
    }

    fn set_offset(&mut self, x: i16, y: i16) {
        self.offset = GridPosition::from((self.position.x + x, self.position.y + y));
        //self.rect = graphics::Rect::from(self.position);

        /*self.outer_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), 
            self.rect, COLOR_ORANGE_DARK).unwrap();

        self.inner_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), 
            self.inner_rect(), COLOR_ORANGE_LIGHT).unwrap();*/
    }
}

#[derive(Clone, Copy, Debug)]
enum PieceKind {
    SQUARE,
    TALL,
}

#[derive(Clone, Debug)]
struct Piece {
    position: GridPosition,
    kind: PieceKind,
    color: BlockColor,
    blocks: Vec<Vec<Block>>,
    active: bool,
}

impl Piece {
    fn new(ctx: &mut Context, x: i16, y: i16, kind: PieceKind, color: BlockColor) -> Piece {
        let mut p = Piece {
            position: GridPosition::from((x, y)),
            kind: kind,
            color: color,
            blocks: Vec::with_capacity(4),
            active: true,
        };

        let row = vec![Block::empty(ctx, color); 4];
        for _ in 0..4 {
            p.blocks.push(row.clone());
        }

        match kind {
            PieceKind::SQUARE => {
                p.blocks[0][0].activate();
                p.blocks[0][1].activate();
                p.blocks[1][0].activate();
                p.blocks[1][1].activate();

                
            },
            PieceKind::TALL => {
                p.blocks[0][0].activate();
                p.blocks[0][1].activate();
                p.blocks[0][2].activate();
                p.blocks[0][3].activate();
            }
        }

        for r in 0..4 {
            for c in 0..4 {
                p.blocks[r][c].set_offset(r as i16, c as i16);
            }
        }
        
        p
    }

    fn new_random(ctx: &mut Context) -> Piece {
        let p = Piece::new(ctx, 4, 0, PieceKind::TALL, BlockColor::RED);
        p
    }

    fn update(&mut self, ctx: &mut Context) {
        if !self.active{ return; }

        self.position.y += 1;
        for r in 0..4 {
            for c in 0..4 {
                self.blocks[r][c].update(ctx, self.position);

                if self.blocks[r][c].position.y + 1 == GRID_COLS as i16 {
                    self.active = false;
                }
            }
        }
        println!("x: {} , y: {}", self.position.x, self.position.y);
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for r in 0..4 {
            for c in 0..4 {
                self.blocks[r][c].draw(ctx)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct GridCell {
    position: GridPosition,
    rect: graphics::Rect,
    occupied: bool
}

impl GridCell {
    fn new() -> GridCell {
        GridCell {
            position: (0, 0).into(),
            rect: graphics::Rect::zero(),
            occupied: false,
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let rectangle = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(2.0),
            self.rect, COLOR_WHITE).unwrap();

        graphics::draw(ctx, &rectangle, graphics::DrawParam::default()).unwrap();

        Ok(())
    }

    fn set_position(&mut self, x: i16, y: i16) {
        self.position = (x, y).into();
        self.rect = self.position.into();
    }
}

struct Grid {
    x: f32,
    y: f32,
    cells: Vec<Vec<GridCell>>
}

impl Grid {
    pub fn new(x: f32, y: f32) -> Grid {
        let mut grid = Grid {
            x: x,
            y: y,
            cells: Vec::with_capacity(GRID_ROWS),
        };

        let row = vec![GridCell::new(); GRID_COLS];
        for _ in 0..GRID_ROWS {
            grid.cells.push(row.clone());
        }

        for r in 0..GRID_ROWS {
            for c in 0..GRID_COLS {
                grid.cells[r][c].set_position(r as i16, c as i16);
            }
        }

        grid
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for r in 0..GRID_ROWS {
            for c in 0..GRID_COLS {
                self.cells[r][c].draw(ctx).unwrap();
            }
        }

        Ok(())
    }
}

struct State {
    dt: std::time::Duration,
    last_update: Instant,
    piece: Piece,
    grid: Grid,
}

impl State {
    pub fn new(ctx: &mut Context) -> State {
        // Initialization code here
        State {
            dt: std::time::Duration::new(0, 0),
            last_update: Instant::now(),
            piece: Piece::new_random(ctx),
            grid: Grid::new(GRID_POS_X, GRID_POS_Y),
        }
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dt = timer::delta(ctx);

        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.piece.update(ctx);

            self.last_update = Instant::now();
        }
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.1, 0.1, 0.1, 1.0].into());

        self.grid.draw(ctx)?;
        self.piece.draw(ctx)?;
        
        graphics::present(ctx)?;
        Ok(())
  }
}

fn main() {
    println!("Hello, world!");

    let c = conf::Conf::new();
    
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("rustycubes", "Dylan Socolobsky")
    .conf(c).build().unwrap();

    ggez::graphics::set_mode(ctx, ggez::conf::WindowMode{
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: WINDOW_WIDTH,
        min_height: WINDOW_HEIGHT,
        max_width: WINDOW_WIDTH,
        max_height: WINDOW_HEIGHT,
        resizable: false
    }).unwrap();

    ggez::graphics::set_window_title(ctx, "RustyCubes - 0.1.0");

    ggez::graphics::set_screen_coordinates(ctx, ggez::graphics::Rect::new(0.0, 0.0, WINDOW_WIDTH, WINDOW_HEIGHT))
        .unwrap();

    let state = &mut State::new(ctx);

    event::run(ctx, event_loop, state).unwrap();
}
