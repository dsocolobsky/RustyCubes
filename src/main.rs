use ggez::*;
use ggez::graphics::Color;
use std::time::{Duration, Instant};
use rand::distributions::{Distribution, Standard};
use rand::Rng;

const COLOR_CYAN_LIGHT: Color = Color {r: 50.0/255.0, g: 200.0/255.0, b: 240.0/255.0, a: 1.0};
const COLOR_CYAN_DARK: Color = Color {r: 25.0/255.0, g: 175.0/255.0, b: 215.0/255.0, a: 1.0};

const COLOR_BLUE_LIGHT: Color = Color {r: 108.0/255.0, g: 125.0/255.0, b: 200.0/255.0, a: 1.0};
const COLOR_BLUE_DARK: Color = Color {r: 70.0/255.0, g: 85.0/255.0, b: 160.0/255.0, a: 1.0};

const COLOR_ORANGE_LIGHT: Color = Color {r: 255.0/255.0, g: 140.0/255.0, b: 55.0/255.0, a: 1.0};
const COLOR_ORANGE_DARK: Color = Color {r: 225.0/255.0, g: 105.0/255.0, b: 20.0/255.0, a: 1.0};

const COLOR_YELLOW_LIGHT: Color = Color {r: 255.0/255.0, g: 232.0/255.0, b: 25.0/255.0, a: 1.0};
const COLOR_YELLOW_DARK: Color = Color {r: 230.0/255.0, g: 195.0/255.0, b: 0.0/255.0, a: 1.0};

const COLOR_GREEN_LIGHT: Color = Color {r: 80.0/255.0, g: 200.0/255.0, b: 80.0/255.0, a: 1.0};
const COLOR_GREEN_DARK: Color = Color {r: 45.0/255.0, g: 165.0/255.0, b: 45.0/255.0, a: 1.0};

const COLOR_PURPLE_LIGHT: Color = Color {r: 195.0/255.0, g: 92.0/255.0, b: 175.0/255.0, a: 1.0};
const COLOR_PURPLE_DARK: Color = Color {r: 150.0/255.0, g: 60.0/255.0, b: 135.0/255.0, a: 1.0};

const COLOR_RED_LIGHT: Color = Color {r: 255.0/255.0, g: 65.0/255.0, b: 70.0/255.0, a: 1.0};
const COLOR_RED_DARK: Color = Color {r: 215.0/255.0, g: 20.0/255.0, b: 25.0/255.0, a: 1.0};


const COLOR_WHITE: Color = ggez::graphics::WHITE;

// Here we're defining how many quickly we want our game to update. This will be
// important later so that we don't have our snake fly across the screen because
// it's moving a full tile every frame.
const UPDATES_PER_SECOND: f32 = 6.0;
// And we get the milliseconds of delay that this update rate corresponds to.
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

const BLOCK_SIZE: f32 = 32.0;
const BLOCK_INNER_SIZE: f32 = BLOCK_SIZE - 1.0;

// grid[rows][cols]
const GRID_ROWS: usize = 10;
const GRID_COLS: usize = 20;
const GRID_SIZE: f32 = BLOCK_SIZE + 4.0;
const GRID_POS_X: f32 = 250.0;
const GRID_POS_Y: f32 = 80.0;

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 920.0;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    LEFT,
    RIGHT
}

#[derive(Clone, Copy, Debug)]
enum PieceKind {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

fn color_for_kind(kind: PieceKind) -> (Color, Color) {
    match kind {
        PieceKind::I => (COLOR_CYAN_DARK, COLOR_CYAN_LIGHT),
        PieceKind::J => (COLOR_BLUE_DARK, COLOR_BLUE_LIGHT),
        PieceKind::L => (COLOR_ORANGE_DARK, COLOR_ORANGE_LIGHT),
        PieceKind::O => (COLOR_YELLOW_DARK, COLOR_YELLOW_LIGHT),
        PieceKind::S => (COLOR_GREEN_DARK, COLOR_GREEN_LIGHT),
        PieceKind::T => (COLOR_PURPLE_DARK, COLOR_PURPLE_LIGHT),
        PieceKind::Z => (COLOR_RED_DARK, COLOR_RED_LIGHT),
    }
}

#[derive(Clone, Debug)]
struct Block {
    kind: PieceKind,
    position: GridPosition,
    offset: GridPosition,
    rect: graphics::Rect,
    outer_mesh: graphics::Mesh,
    inner_mesh: graphics::Mesh,
    active: bool,
    render: bool,
}

impl Block {
    pub fn new(ctx: &mut Context, x: i16, y: i16, kind: PieceKind) -> Block {
        let position = GridPosition::from((x, y));
        let rect = graphics::Rect::from(position);

        let (dark_color, light_color) = color_for_kind(kind);

        let block = Block {
            kind: kind,
            position: position,
            rect: rect,
            offset: (0, 0).into(),
            outer_mesh: graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), 
                    rect, dark_color).unwrap(),
            inner_mesh: graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), 
                graphics::Rect::new(rect.x+2.0, rect.y+2.0, BLOCK_INNER_SIZE, BLOCK_INNER_SIZE),
                light_color).unwrap(),
            active: false,
            render: false,
        };

        block
    }

    pub fn empty(ctx: &mut Context, kind: PieceKind) -> Block {
        Block::new(ctx, 0, 0, kind)
    }

    fn update(&mut self, ctx: &mut Context, parent_position: GridPosition) {
        if !self.active { return; }

        self.position.x = parent_position.x + self.offset.x;
        self.position.y = parent_position.y + self.offset.y;

        self.rect = graphics::Rect::from(self.position);

        let (dark_color, light_color) = color_for_kind(self.kind);

        self.outer_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(),
            self.rect, dark_color).unwrap();

        self.inner_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(),
            self.inner_rect(), light_color).unwrap();
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        if self.render {
            graphics::draw(ctx, &self.outer_mesh, graphics::DrawParam::default()).unwrap();
            graphics::draw(ctx, &self.inner_mesh, graphics::DrawParam::default()).unwrap();
        }
        Ok(())
    }

    fn inner_rect(&self) -> graphics::Rect {
        graphics::Rect::new(self.rect.x+2.0, self.rect.y+2.0, BLOCK_INNER_SIZE, BLOCK_INNER_SIZE)
    }

    fn do_render(&mut self) {
        self.render = true;
    }

    fn set_inactive(&mut self) {
        self.active = false;
    }

    fn active_and_render(&mut self) {
        self.render = true;
        self.active = true;
    }

    fn set_offset(&mut self, x: i16, y: i16) {
        self.offset = GridPosition::from((self.position.x + x, self.position.y + y));
    }
}



impl Distribution<PieceKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PieceKind { // I J L O S T Z
        match rng.gen_range(0..7) {
            0 => PieceKind::I,
            1 => PieceKind::J,
            2 => PieceKind::L,
            3 => PieceKind::O,
            4 => PieceKind::S,
            5 => PieceKind::T,
            _ => PieceKind::Z,
        }
    }
}

#[derive(Clone, Debug)]
struct Piece {
    position: GridPosition,
    kind: PieceKind,
    blocks: Vec<Vec<Block>>,
    active: bool,
}

impl Piece {
    fn new(ctx: &mut Context, x: i16, y: i16, kind: PieceKind) -> Piece {
        let mut p = Piece {
            position: GridPosition::from((x, y)),
            kind: kind,
            blocks: Vec::with_capacity(4),
            active: true,
        };

        let row = vec![Block::empty(ctx, kind); 4];
        for _ in 0..4 {
            p.blocks.push(row.clone());
        }

        match kind {
            PieceKind::I => {
                p.blocks[0][0].active_and_render();
                p.blocks[1][0].active_and_render();
                p.blocks[2][0].active_and_render();
                p.blocks[3][0].active_and_render();
            },
            PieceKind::J => {
                p.blocks[0][0].active_and_render();
                p.blocks[0][1].active_and_render();
                p.blocks[1][1].active_and_render();
                p.blocks[2][1].active_and_render();
            },
            PieceKind::L => {
                p.blocks[2][0].active_and_render();
                p.blocks[0][1].active_and_render();
                p.blocks[1][1].active_and_render();
                p.blocks[2][1].active_and_render();
            },
            PieceKind::O => {
                p.blocks[0][0].active_and_render();
                p.blocks[0][1].active_and_render();
                p.blocks[1][0].active_and_render();
                p.blocks[1][1].active_and_render();
            },
            PieceKind::S => {
                p.blocks[1][0].active_and_render();
                p.blocks[2][0].active_and_render();
                p.blocks[0][1].active_and_render();
                p.blocks[1][1].active_and_render();
            },
            PieceKind::T => {
                p.blocks[1][0].active_and_render();
                p.blocks[0][1].active_and_render();
                p.blocks[1][1].active_and_render();
                p.blocks[2][1].active_and_render();
            },
            PieceKind::Z => {
                p.blocks[0][0].active_and_render();
                p.blocks[1][0].active_and_render();
                p.blocks[1][1].active_and_render();
                p.blocks[2][1].active_and_render();
            },
        }

        for r in 0..4 {
            for c in 0..4 {
                p.blocks[r][c].set_offset(r as i16, c as i16);
            }
        }
        
        p
    }

    fn new_random(ctx: &mut Context) -> Piece {
        let kind: PieceKind = rand::random();
        let p = Piece::new(ctx, 4, 0, kind);
        p
    }

    fn update_fast(&mut self, ctx: &mut Context) {
        for r in 0..4 {
            for c in 0..4 {
                self.blocks[r][c].update(ctx, self.position);

                if self.blocks[r][c].position.y + 1 == GRID_COLS as i16 {
                    self.active = false;
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut Context, grid: &mut Grid) {
        if !self.active{ return; }

        self.position.y += 1;
        for r in 0..4 {
            for c in 0..4 {
                let b = &mut self.blocks[r][c];
                if !b.active { continue; }

                b.update(ctx, self.position);
                if b.position.y + 1 == GRID_COLS as i16 {
                    self.active = false;
                }
                
                if b.position.x < 10 && b.position.y < 19 && grid.cells[b.position.x as usize][b.position.y as usize + 1].occupied {
                    println!("OCCUPIED");
                    self.active = false;
                }
            }
        }

        // Piece is now dead
        if !self.active {
            for r in 0..4 {
                for c in 0..4 {
                    if !self.blocks[r][c].render {continue;}

                    let mut pos = self.blocks[r][c].position;
                    println!("Dying at {}, {}", pos.x, pos.y);
                    pos.x = if pos.x >= GRID_ROWS as i16 { GRID_ROWS as i16 - 1 } else { pos.x };
                    pos.y = if pos.y >= GRID_COLS as i16 { GRID_COLS as i16 - 1 } else { pos.y };
                    println!("Actualized to {}, {}", pos.x, pos.y);

                    grid.cells[pos.x as usize][pos.y as usize].set_block(ctx, &self.blocks[r][c]);
                }
            }
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for r in 0..4 {
            for c in 0..4 {
                self.blocks[r][c].draw(ctx)?;
            }
        }
        Ok(())
    }

    fn move_left(&mut self) {
        self.position.x -= 1;
    }

    fn move_right(&mut self) {
        self.position.x += 1;
    }
}

#[derive(Debug, Clone)]
struct GridCell {
    position: GridPosition,
    rect: graphics::Rect,
    occupied: bool,
    block: Option<Block>
}

impl GridCell {
    fn new() -> GridCell {
        GridCell {
            position: (0, 0).into(),
            rect: graphics::Rect::zero(),
            occupied: false,
            block: None
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let rectangle = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(2.0),
            self.rect, COLOR_WHITE).unwrap();

        graphics::draw(ctx, &rectangle, graphics::DrawParam::default()).unwrap();

        if let Some(block) = &self.block {
            block.draw(ctx)?;
        }

        Ok(())
    }

    fn set_position(&mut self, x: i16, y: i16) {
        self.position = (x, y).into();
        self.rect = self.position.into();
    }

    fn set_block(&mut self, ctx: &mut Context, block: &Block) {
        println!("Setting block with x: {}, y: {}", block.position.x, block.position.y);
        self.occupied = true;
        self.block = Some(Block::new(ctx, block.position.x, block.position.y, block.kind));
        if let Some(block) = &mut self.block {
            block.set_inactive();
            block.do_render();
        }
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
    piece: Option<Piece>,
    grid: Grid,
}

impl State {
    pub fn new(ctx: &mut Context) -> State {
        // Initialization code here
        State {
            dt: std::time::Duration::new(0, 0),
            last_update: Instant::now(),
            piece: Some(Piece::new_random(ctx)),
            grid: Grid::new(GRID_POS_X, GRID_POS_Y),
        }
    }

    
}

fn can_move(piece: &Piece, grid: &Grid, direction: Direction) -> bool {
    println!("=====================================================");
    if direction == Direction::RIGHT && (piece.position.x >= GRID_ROWS as i16 - 1) {
        return false;
    } else if direction == Direction::LEFT && piece.position.x <= 0 {
        return false;
    }
    
    for r in 0..4 {
        for c in 0..4 {
            if !piece.blocks[r][c].active { continue; }
            let pos = piece.blocks[r][c].position;
            println!("[x: {}, y: {}]", pos.x, pos.y);
            let py = pos.y as usize;
            if direction == Direction::LEFT {
                if pos.x == 0 {
                    println!("px is 0 !");
                    return false
                }
                let px = (pos.x - 1) as usize;
                if grid.cells[px][py].occupied {
                    println!("{} is occupied!", px);
                    return false
                }
            } else {
                let px = (pos.x + 1) as usize;
                if px == 10 || grid.cells[px][py].occupied {
                    return false
                }
            }
        }
    }
    println!("=====================================================");
    true
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dt = timer::delta(ctx);

        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            if let Some(piece) = &mut self.piece {
                piece.update(ctx, &mut self.grid);

                if !piece.active {
                    self.piece = Some(Piece::new_random(ctx));
                }
            }
            

            self.last_update = Instant::now();
        } else {
            if let Some(piece) = &mut self.piece {
                piece.update_fast(ctx);
            }
        }
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.1, 0.1, 0.1, 1.0].into());

        self.grid.draw(ctx)?;

        if let Some(piece) = &self.piece {
            piece.draw(ctx)?;
        }
        
        graphics::present(ctx)?;
        Ok(())
  }

  fn key_down_event(&mut self, ctx: &mut Context, keycode: ggez::event::KeyCode, _keymods: ggez::event::KeyMods, _repeat: bool) {
    match keycode {
        ggez::event::KeyCode::Right => {
            if let Some(piece) = &mut self.piece {
                if can_move(&piece, &self.grid, Direction::RIGHT) {
                    piece.move_right();
                }
            }
        },
        ggez::event::KeyCode::Left => {
            if let Some(piece) = &mut self.piece {
                if can_move(&piece, &self.grid, Direction::LEFT) {
                    piece.move_left();
                }
            }
        },
        _ => {}
    }
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
