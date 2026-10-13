pub const HEIGHT: f32 = 4.0;

#[derive(Clone, Copy)]
pub struct Paddle {
    pub column: u16,
    pub top: f32,
    floor: f32,
}

impl Paddle {
    pub fn new(column: u16, court_rows: u16) -> Self {
        let floor = f32::from(court_rows) - HEIGHT;
        Self {
            column,
            top: floor / 2.0,
            floor,
        }
    }

    pub fn center(&self) -> f32 {
        self.top + HEIGHT / 2.0
    }

    pub fn shift(&mut self, rows: f32) {
        self.top = (self.top + rows).clamp(0.0, self.floor);
    }

    pub fn covers(&self, y: f32) -> bool {
        (self.top - 0.5..self.top + HEIGHT + 0.5).contains(&y)
    }

    /// Where along the paddle the ball hit, from -1 (top edge) to 1 (bottom edge).
    pub fn hit_offset(&self, y: f32) -> f32 {
        ((y - self.center()) / (HEIGHT / 2.0)).clamp(-1.0, 1.0)
    }
}
