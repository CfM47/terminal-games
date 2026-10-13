use std::time::Duration;

use super::paddle::Paddle;
use crate::games::ball::Ball;

const BASE_SPEED: f32 = 9.0;
const SPEED_PER_MATCH: f32 = 2.5;
const MAX_SPEED: f32 = 22.0;
const BASE_AIM_ERROR: f32 = 2.5;
const AIM_ERROR_DROP_PER_MATCH: f32 = 0.4;
const MIN_AIM_ERROR: f32 = 0.3;

/// Computer opponent that chases the ball with limited speed and a deliberate aiming error.
pub struct Cpu {
    speed: f32,
    aim_error: f32,
    aim_offset: f32,
}

impl Cpu {
    pub fn for_match(number: u32) -> Self {
        let experience = (number - 1) as f32;
        Self {
            speed: (BASE_SPEED + SPEED_PER_MATCH * experience).min(MAX_SPEED),
            aim_error: (BASE_AIM_ERROR - AIM_ERROR_DROP_PER_MATCH * experience).max(MIN_AIM_ERROR),
            aim_offset: 0.0,
        }
    }

    pub fn rethink(&mut self) {
        self.aim_offset = (fastrand::f32() * 2.0 - 1.0) * self.aim_error;
    }

    pub fn steer(&self, paddle: &mut Paddle, ball: &Ball, rest_row: f32, elapsed: Duration) {
        let target = if ball.is_moving_right() {
            ball.y + self.aim_offset
        } else {
            rest_row
        };
        let reach = self.speed * elapsed.as_secs_f32();
        paddle.shift((target - paddle.center()).clamp(-reach, reach));
    }
}
