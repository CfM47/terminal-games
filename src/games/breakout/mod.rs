mod ball;
mod sounds;
mod wall;

use std::time::Duration;

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Widget},
};

use self::{
    ball::Ball,
    wall::{BRICK_WIDTH, Wall},
};
use crate::{audio::Speaker, game::Game, input::HeldKeys};

const COLUMNS: u16 = wall::COLUMNS as u16 * BRICK_WIDTH;
const ROWS: u16 = 24;
const PADDLE_ROW: u16 = ROWS - 2;
const PADDLE_WIDTH: u16 = 8;
const PADDLE_SPEED: f32 = 48.0;
const STARTING_LIVES: u32 = 3;
const BASE_BALL_SPEED: f32 = 14.0;
const BALL_SPEEDUP_PER_LEVEL: f32 = 2.0;
const BALL_SPEEDUP_PER_HIT: f32 = 0.3;
const MAX_BALL_SPEED: f32 = 28.0;
const MAX_SERVE_ANGLE: f32 = 0.4;
const MAX_STEP_CELLS: f32 = 0.25;
const LEFT_KEYS: [KeyCode; 2] = [KeyCode::Left, KeyCode::Char('a')];
const RIGHT_KEYS: [KeyCode; 2] = [KeyCode::Right, KeyCode::Char('d')];

pub struct Breakout {
    wall: Wall,
    ball: Option<Ball>,
    paddle_left: f32,
    held: HeldKeys,
    lives: u32,
    level: u32,
    score: u32,
}

impl Breakout {
    pub fn new() -> Self {
        Self {
            wall: Wall::new(),
            ball: None,
            paddle_left: f32::from(COLUMNS - PADDLE_WIDTH) / 2.0,
            held: HeldKeys::default(),
            lives: STARTING_LIVES,
            level: 1,
            score: 0,
        }
    }

    fn paddle_center(&self) -> f32 {
        self.paddle_left + f32::from(PADDLE_WIDTH) / 2.0
    }

    fn serve(&mut self, speaker: &Speaker) {
        if self.ball.is_some() {
            return;
        }
        let angle = (fastrand::f32() * 2.0 - 1.0) * MAX_SERVE_ANGLE;
        let speed = BASE_BALL_SPEED + BALL_SPEEDUP_PER_LEVEL * (self.level - 1) as f32;
        let height = f32::from(PADDLE_ROW) - 0.5;
        self.ball = Some(Ball::launched(self.paddle_center(), height, speed, angle));
        speaker.play(&sounds::PADDLE);
    }

    fn steer_paddle(&mut self, elapsed: Duration) {
        let direction = match (
            self.held.is_held(&LEFT_KEYS),
            self.held.is_held(&RIGHT_KEYS),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let travel = direction * PADDLE_SPEED * elapsed.as_secs_f32();
        self.paddle_left =
            (self.paddle_left + travel).clamp(0.0, f32::from(COLUMNS - PADDLE_WIDTH));
    }

    fn advance_ball(&mut self, ball: Ball, elapsed: Duration, speaker: &Speaker) -> Option<Ball> {
        let (dx, dy) = ball.travel(elapsed.as_secs_f32());
        let steps = (dx.abs().max(dy.abs()) / MAX_STEP_CELLS).ceil().max(1.0);
        let step = elapsed.div_f32(steps);
        (0..steps as u32).try_fold(ball, |ball, _| self.step_ball(ball, step, speaker))
    }

    /// Moves the ball one small step, resolving each axis separately; returns `None` once it is lost.
    fn step_ball(&mut self, mut ball: Ball, step: Duration, speaker: &Speaker) -> Option<Ball> {
        let (dx, dy) = ball.travel(step.as_secs_f32());

        let x = ball.x + dx;
        if !(0.0..f32::from(COLUMNS)).contains(&x) {
            ball.bounce_horizontally();
            speaker.play(&sounds::WALL);
        } else if let Some(points) = self.wall.strike(x, ball.y) {
            ball.bounce_horizontally();
            self.score_brick(points, speaker);
        } else {
            ball.x = x;
        }

        let y = ball.y + dy;
        if y < 0.0 {
            ball.bounce_vertically();
            speaker.play(&sounds::WALL);
        } else if let Some(points) = self.wall.strike(ball.x, y) {
            ball.bounce_vertically();
            self.score_brick(points, speaker);
        } else if ball.is_falling() && self.paddle_catches(ball.x, ball.y, y) {
            let offset = (ball.x - self.paddle_center()) / (f32::from(PADDLE_WIDTH) / 2.0);
            ball.deflect(offset, BALL_SPEEDUP_PER_HIT, MAX_BALL_SPEED);
            speaker.play(&sounds::PADDLE);
        } else if y >= f32::from(ROWS) {
            return None;
        } else {
            ball.y = y;
        }
        Some(ball)
    }

    fn paddle_catches(&self, x: f32, from_y: f32, to_y: f32) -> bool {
        let row = f32::from(PADDLE_ROW);
        let reach = self.paddle_left - 0.5..self.paddle_left + f32::from(PADDLE_WIDTH) + 0.5;
        from_y < row && to_y >= row && reach.contains(&x)
    }

    fn score_brick(&mut self, points: u32, speaker: &Speaker) {
        self.score += points;
        speaker.play(&sounds::BRICK);
    }

    fn lose_life(&mut self, speaker: &Speaker) {
        self.lives -= 1;
        if self.lives > 0 {
            speaker.play(&sounds::LOST_BALL);
        }
    }

    fn advance_level(&mut self, speaker: &Speaker) {
        self.level += 1;
        self.wall = Wall::new();
        self.ball = None;
        speaker.play(&sounds::WALL_CLEARED);
    }

    fn frame_block(&self) -> Block<'static> {
        let hearts = "♥ ".repeat(self.lives as usize);
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray())
            .title_top(Line::from(format!(" {hearts}")).light_red())
            .title_top(Line::from(format!(" Level {} ", self.level)).right_aligned())
    }

    fn draw_field(&self, buffer: &mut Buffer, field: Rect) {
        for brick in self.wall.bricks() {
            let style = Style::new().fg(brick.color);
            buffer.set_string(field.x + brick.x, field.y + brick.y, "▐██▌", style);
        }

        let paddle = "█".repeat(PADDLE_WIDTH as usize);
        let paddle_x = field.x + self.paddle_left.round() as u16;
        buffer.set_string(paddle_x, field.y + PADDLE_ROW, paddle, Style::new().white());

        let (ball_x, ball_y) = match self.ball {
            Some(ball) => (ball.x, ball.y),
            None => (self.paddle_center(), f32::from(PADDLE_ROW - 1)),
        };
        let ball_style = Style::new().fg(Color::LightYellow);
        buffer.set_string(
            field.x + ball_x as u16,
            field.y + ball_y as u16,
            "●",
            ball_style,
        );
    }
}

impl Game for Breakout {
    fn size(&self) -> Size {
        Size::new(COLUMNS + 2, ROWS + 2)
    }

    fn handle_key(&mut self, key: KeyCode, speaker: &Speaker) {
        match key {
            KeyCode::Char(' ') | KeyCode::Up | KeyCode::Char('w') => self.serve(speaker),
            _ => self.held.press(key),
        }
    }

    fn handle_key_release(&mut self, key: KeyCode) {
        self.held.release(key);
    }

    fn update(&mut self, elapsed: Duration, speaker: &Speaker) {
        self.steer_paddle(elapsed);
        self.held.tick(elapsed);
        let Some(ball) = self.ball else { return };
        self.ball = self.advance_ball(ball, elapsed, speaker);
        if self.ball.is_none() {
            self.lose_life(speaker);
        } else if self.wall.is_cleared() {
            self.advance_level(speaker);
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = self.frame_block();
        let field = block.inner(area);
        block.render(area, frame.buffer_mut());
        self.draw_field(frame.buffer_mut(), field);
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn is_over(&self) -> bool {
        self.lives == 0
    }
}
