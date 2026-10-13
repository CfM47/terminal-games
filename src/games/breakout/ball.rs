use std::f32::consts::FRAC_PI_3;

/// Terminal cells are about twice as tall as they are wide.
const CELL_ASPECT: f32 = 2.0;
const MAX_BOUNCE_ANGLE: f32 = FRAC_PI_3;

#[derive(Clone, Copy)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    heading_x: f32,
    heading_y: f32,
    speed: f32,
}

impl Ball {
    pub fn launched(x: f32, y: f32, speed: f32, angle: f32) -> Self {
        let mut ball = Self {
            x,
            y,
            heading_x: 0.0,
            heading_y: -1.0,
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

    pub fn is_falling(&self) -> bool {
        self.heading_y > 0.0
    }

    pub fn bounce_horizontally(&mut self) {
        self.heading_x = -self.heading_x;
    }

    pub fn bounce_vertically(&mut self) {
        self.heading_y = -self.heading_y;
    }

    /// Sends the ball upward at an angle set by where it hit the paddle, from -1 (left edge) to 1.
    pub fn deflect(&mut self, offset: f32, speedup: f32, max_speed: f32) {
        self.aim(offset.clamp(-1.0, 1.0) * MAX_BOUNCE_ANGLE);
        self.speed = (self.speed + speedup).min(max_speed);
    }

    fn aim(&mut self, angle: f32) {
        self.heading_x = angle.sin();
        self.heading_y = -angle.cos();
    }
}
