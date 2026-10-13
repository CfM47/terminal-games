use std::time::Duration;

/// Terminal cells are about twice as tall as they are wide.
const CELL_ASPECT: f32 = 2.0;
const MAX_STEP_CELLS: f32 = 0.25;

/// Ball moving at a constant on-screen speed, with angles measured clockwise from the right.
#[derive(Clone, Copy)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    heading_x: f32,
    heading_y: f32,
    speed: f32,
}

impl Ball {
    pub fn new(x: f32, y: f32, speed: f32, angle: f32) -> Self {
        let mut ball = Self {
            x,
            y,
            heading_x: 1.0,
            heading_y: 0.0,
            speed,
        };
        ball.aim(angle);
        ball
    }

    /// Displacement in cells over the given number of seconds.
    pub fn travel(&self, seconds: f32) -> (f32, f32) {
        let distance = self.speed * seconds;
        (
            self.heading_x * distance * CELL_ASPECT,
            self.heading_y * distance,
        )
    }

    /// Splits a frame into steps short enough that the ball never skips a cell.
    pub fn substeps(&self, elapsed: Duration) -> impl Iterator<Item = Duration> + use<> {
        let (dx, dy) = self.travel(elapsed.as_secs_f32());
        let count = (dx.abs().max(dy.abs()) / MAX_STEP_CELLS).ceil().max(1.0);
        std::iter::repeat_n(elapsed.div_f32(count), count as usize)
    }

    pub fn is_moving_down(&self) -> bool {
        self.heading_y > 0.0
    }

    pub fn is_moving_right(&self) -> bool {
        self.heading_x > 0.0
    }

    pub fn bounce_horizontally(&mut self) {
        self.heading_x = -self.heading_x;
    }

    pub fn bounce_vertically(&mut self) {
        self.heading_y = -self.heading_y;
    }

    pub fn aim(&mut self, angle: f32) {
        self.heading_x = angle.cos();
        self.heading_y = angle.sin();
    }

    pub fn accelerate(&mut self, amount: f32, max_speed: f32) {
        self.speed = (self.speed + amount).min(max_speed);
    }
}
