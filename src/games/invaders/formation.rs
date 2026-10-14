use ratatui::style::Color;

pub const SPRITE_WIDTH: u16 = 3;
const COLUMNS: usize = 11;
const ROWS: usize = 5;
const COLUMN_PITCH: u16 = SPRITE_WIDTH + 1;
const ROW_PITCH: u16 = 2;
const START_LEFT: i16 = 2;

#[derive(Clone, Copy)]
pub enum Kind {
    Squid,
    Crab,
    Octopus,
}

impl Kind {
    fn for_row(row: usize) -> Self {
        match row {
            0 => Kind::Squid,
            1 | 2 => Kind::Crab,
            _ => Kind::Octopus,
        }
    }

    pub fn points(self) -> u32 {
        match self {
            Kind::Squid => 30,
            Kind::Crab => 20,
            Kind::Octopus => 10,
        }
    }

    pub fn sprite(self, legs_out: bool) -> &'static str {
        match (self, legs_out) {
            (Kind::Squid, false) => "/O\\",
            (Kind::Squid, true) => "\\O/",
            (Kind::Crab, false) => "{#}",
            (Kind::Crab, true) => "}#{",
            (Kind::Octopus, false) => "<W>",
            (Kind::Octopus, true) => ">W<",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Kind::Squid => Color::LightMagenta,
            Kind::Crab => Color::LightCyan,
            Kind::Octopus => Color::LightGreen,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Invader {
    pub x: u16,
    pub y: u16,
    pub kind: Kind,
}

impl Invader {
    pub fn covers(&self, x: u16, y: u16) -> bool {
        y == self.y && (self.x..self.x + SPRITE_WIDTH).contains(&x)
    }

    pub fn center(&self) -> u16 {
        self.x + SPRITE_WIDTH / 2
    }
}

/// The marching block of invaders, moving one column per step and dropping a row at each edge.
pub struct Formation {
    alive: [[bool; COLUMNS]; ROWS],
    left: i16,
    top: u16,
    marching_right: bool,
    pub legs_out: bool,
}

impl Formation {
    pub const SIZE: usize = COLUMNS * ROWS;

    pub fn new(top: u16) -> Self {
        Self {
            alive: [[true; COLUMNS]; ROWS],
            left: START_LEFT,
            top,
            marching_right: true,
            legs_out: false,
        }
    }

    pub fn invaders(&self) -> impl Iterator<Item = Invader> + '_ {
        self.alive
            .iter()
            .enumerate()
            .flat_map(move |(row, columns)| {
                columns
                    .iter()
                    .enumerate()
                    .filter(|(_, alive)| **alive)
                    .map(move |(column, _)| self.invader(row, column))
            })
    }

    pub fn remaining(&self) -> usize {
        self.alive.iter().flatten().filter(|alive| **alive).count()
    }

    pub fn lowest_row(&self) -> Option<u16> {
        self.invaders().map(|invader| invader.y).max()
    }

    /// Kills the invader covering the cell, if any.
    pub fn strike(&mut self, x: u16, y: u16) -> Option<Invader> {
        let hit = self.invaders().find(|invader| invader.covers(x, y))?;
        let row = usize::from((hit.y - self.top) / ROW_PITCH);
        let column = ((i32::from(hit.x) - i32::from(self.left)) / i32::from(COLUMN_PITCH)) as usize;
        self.alive[row][column] = false;
        Some(hit)
    }

    /// Bottom invader of a random column, the only ones allowed to drop bombs.
    pub fn random_shooter(&self) -> Option<Invader> {
        let shooters: Vec<Invader> = (0..COLUMNS)
            .filter_map(|column| {
                (0..ROWS)
                    .rev()
                    .find(|&row| self.alive[row][column])
                    .map(|row| self.invader(row, column))
            })
            .collect();
        fastrand::choice(shooters)
    }

    pub fn advance(&mut self, field_width: u16) {
        self.legs_out = !self.legs_out;
        let xs = || self.invaders().map(|invader| invader.x);
        let (Some(leftmost), Some(rightmost)) = (xs().min(), xs().max()) else {
            return;
        };
        let blocked = if self.marching_right {
            rightmost + SPRITE_WIDTH >= field_width
        } else {
            leftmost == 0
        };
        if blocked {
            self.top += 1;
            self.marching_right = !self.marching_right;
        } else {
            self.left += if self.marching_right { 1 } else { -1 };
        }
    }

    fn invader(&self, row: usize, column: usize) -> Invader {
        Invader {
            x: (self.left + column as i16 * COLUMN_PITCH as i16) as u16,
            y: self.top + row as u16 * ROW_PITCH,
            kind: Kind::for_row(row),
        }
    }
}
