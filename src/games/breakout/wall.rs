use ratatui::style::Color;

pub const BRICK_WIDTH: u16 = 4;
pub const COLUMNS: usize = 12;
const TOP: u16 = 3;
const ROW_COLORS: [Color; 6] = [
    Color::LightRed,
    Color::Indexed(208),
    Color::LightYellow,
    Color::LightGreen,
    Color::LightCyan,
    Color::LightBlue,
];
const ROWS: usize = ROW_COLORS.len();
const TOP_ROW_POINTS: u32 = 70;
const POINTS_STEP: u32 = 10;

pub struct Brick {
    pub x: u16,
    pub y: u16,
    pub color: Color,
}

pub struct Wall {
    standing: [[bool; COLUMNS]; ROWS],
}

impl Wall {
    pub fn new() -> Self {
        Self {
            standing: [[true; COLUMNS]; ROWS],
        }
    }

    /// Knocks out the brick covering the cell, returning its points.
    pub fn strike(&mut self, x: f32, y: f32) -> Option<u32> {
        let row: usize = (y.floor() as i32 - TOP as i32).try_into().ok()?;
        let column: usize = (x.floor() as i32)
            .div_euclid(BRICK_WIDTH as i32)
            .try_into()
            .ok()?;
        let standing = self.standing.get_mut(row)?.get_mut(column)?;
        if !*standing {
            return None;
        }
        *standing = false;
        Some(TOP_ROW_POINTS - POINTS_STEP * row as u32)
    }

    pub fn is_cleared(&self) -> bool {
        self.standing.iter().flatten().all(|standing| !standing)
    }

    pub fn bricks(&self) -> impl Iterator<Item = Brick> {
        self.standing.iter().zip(0..).flat_map(|(row, y)| {
            row.iter()
                .zip(0..)
                .filter(|(standing, _)| **standing)
                .map(move |(_, x)| Brick {
                    x: x * BRICK_WIDTH,
                    y: TOP + y,
                    color: ROW_COLORS[y as usize],
                })
        })
    }
}
