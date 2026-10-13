mod cpu;
mod digits;
mod paddle;
mod sounds;

use std::{
    f32::consts::{FRAC_PI_3, PI},
    time::Duration,
};

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Widget},
};

use self::{cpu::Cpu, paddle::Paddle};
use super::ball::Ball;
use crate::{audio::Speaker, game::Game, input::HeldKeys};

const COLUMNS: u16 = 60;
const ROWS: u16 = 22;
const LEFT_COLUMN: u16 = 2;
const RIGHT_COLUMN: u16 = COLUMNS - 3;
const NET_COLUMN: u16 = COLUMNS / 2;
const SCORE_TOP: u16 = 1;
const PADDLE_SPEED: f32 = 22.0;
const POINTS_TO_WIN: u32 = 7;
const BALL_START_SPEED: f32 = 16.0;
const BALL_SPEEDUP_PER_HIT: f32 = 0.8;
const MAX_BALL_SPEED: f32 = 32.0;
const MAX_BOUNCE_ANGLE: f32 = FRAC_PI_3;
const MAX_SERVE_ANGLE: f32 = 0.5;
const SERVE_DELAY: Duration = Duration::from_secs(1);
const POINT_SCORE: u32 = 100;
const MATCH_BONUS: u32 = 1000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn opponent(self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

struct Controls {
    up: &'static [KeyCode],
    down: &'static [KeyCode],
}

const SOLO_CONTROLS: Controls = Controls {
    up: &[KeyCode::Up, KeyCode::Char('w')],
    down: &[KeyCode::Down, KeyCode::Char('s')],
};
const LEFT_PLAYER_CONTROLS: Controls = Controls {
    up: &[KeyCode::Char('w')],
    down: &[KeyCode::Char('s')],
};
const RIGHT_PLAYER_CONTROLS: Controls = Controls {
    up: &[KeyCode::Up],
    down: &[KeyCode::Down],
};

enum Mode {
    Solo { cpu: Cpu, match_number: u32 },
    Versus,
}

pub struct Pong {
    mode: Mode,
    left: Paddle,
    right: Paddle,
    ball: Ball,
    serve_in: Duration,
    left_points: u32,
    right_points: u32,
    score: u32,
    held: HeldKeys,
    winner: Option<Side>,
}

impl Pong {
    pub fn solo() -> Self {
        Self::new(Mode::Solo {
            cpu: Cpu::for_match(1),
            match_number: 1,
        })
    }

    pub fn versus() -> Self {
        Self::new(Mode::Versus)
    }

    fn new(mode: Mode) -> Self {
        let mut pong = Self {
            mode,
            left: Paddle::new(LEFT_COLUMN, ROWS),
            right: Paddle::new(RIGHT_COLUMN, ROWS),
            ball: Ball::new(0.0, 0.0, 0.0, 0.0),
            serve_in: Duration::ZERO,
            left_points: 0,
            right_points: 0,
            score: 0,
            held: HeldKeys::default(),
            winner: None,
        };
        pong.serve_toward(if fastrand::bool() {
            Side::Left
        } else {
            Side::Right
        });
        pong
    }

    fn serve_toward(&mut self, side: Side) {
        let spread = (fastrand::f32() * 2.0 - 1.0) * MAX_SERVE_ANGLE;
        let angle = match side {
            Side::Left => PI - spread,
            Side::Right => spread,
        };
        let center = (f32::from(COLUMNS) / 2.0, f32::from(ROWS) / 2.0);
        self.ball = Ball::new(center.0, center.1, BALL_START_SPEED, angle);
        self.serve_in = SERVE_DELAY;
        if let Mode::Solo { cpu, .. } = &mut self.mode {
            cpu.rethink();
        }
    }

    fn steer_paddles(&mut self, elapsed: Duration) {
        let reach = PADDLE_SPEED * elapsed.as_secs_f32();
        match &self.mode {
            Mode::Solo { cpu, .. } => {
                self.left
                    .shift(reach * direction(&self.held, &SOLO_CONTROLS));
                cpu.steer(&mut self.right, &self.ball, f32::from(ROWS) / 2.0, elapsed);
            }
            Mode::Versus => {
                self.left
                    .shift(reach * direction(&self.held, &LEFT_PLAYER_CONTROLS));
                self.right
                    .shift(reach * direction(&self.held, &RIGHT_PLAYER_CONTROLS));
            }
        }
    }

    /// Moves the ball one small step and reports which side scored, if any.
    fn step_ball(&mut self, step: Duration, speaker: &Speaker) -> Option<Side> {
        let (dx, dy) = self.ball.travel(step.as_secs_f32());

        let y = self.ball.y + dy;
        if (0.0..f32::from(ROWS)).contains(&y) {
            self.ball.y = y;
        } else {
            self.ball.bounce_vertically();
            speaker.play(&sounds::WALL);
        }

        let x = self.ball.x + dx;
        if let Some(side) = self.paddle_in_path(self.ball.x, x) {
            self.return_ball(side, speaker);
        } else if x < 0.0 {
            return Some(Side::Right);
        } else if x >= f32::from(COLUMNS) {
            return Some(Side::Left);
        } else {
            self.ball.x = x;
        }
        None
    }

    fn paddle_in_path(&self, from_x: f32, to_x: f32) -> Option<Side> {
        let left_face = f32::from(LEFT_COLUMN + 1);
        let right_face = f32::from(RIGHT_COLUMN);
        let y = self.ball.y;
        if !self.ball.is_moving_right()
            && from_x >= left_face
            && to_x < left_face
            && self.left.covers(y)
        {
            Some(Side::Left)
        } else if self.ball.is_moving_right()
            && from_x < right_face
            && to_x >= right_face
            && self.right.covers(y)
        {
            Some(Side::Right)
        } else {
            None
        }
    }

    fn return_ball(&mut self, side: Side, speaker: &Speaker) {
        let paddle = match side {
            Side::Left => self.left,
            Side::Right => self.right,
        };
        let angle = paddle.hit_offset(self.ball.y) * MAX_BOUNCE_ANGLE;
        self.ball.aim(match side {
            Side::Left => angle,
            Side::Right => PI - angle,
        });
        self.ball.accelerate(BALL_SPEEDUP_PER_HIT, MAX_BALL_SPEED);
        if let (Mode::Solo { cpu, .. }, Side::Left) = (&mut self.mode, side) {
            cpu.rethink();
        }
        speaker.play(&sounds::PADDLE);
    }

    fn award_point(&mut self, scorer: Side, speaker: &Speaker) {
        let points = match scorer {
            Side::Left => &mut self.left_points,
            Side::Right => &mut self.right_points,
        };
        *points += 1;
        let won_match = *points >= POINTS_TO_WIN;
        if matches!(self.mode, Mode::Solo { .. }) && scorer == Side::Left {
            self.score += POINT_SCORE;
        }
        if won_match {
            self.finish_match(scorer, speaker);
        } else {
            speaker.play(&sounds::POINT);
            self.serve_toward(scorer.opponent());
        }
    }

    fn finish_match(&mut self, winner: Side, speaker: &Speaker) {
        let Mode::Solo { cpu, match_number } = &mut self.mode else {
            self.winner = Some(winner);
            return;
        };
        if winner == Side::Right {
            self.winner = Some(winner);
            return;
        }
        self.score += MATCH_BONUS * *match_number;
        *match_number += 1;
        *cpu = Cpu::for_match(*match_number);
        self.left_points = 0;
        self.right_points = 0;
        speaker.play(&sounds::MATCH_WON);
        self.serve_toward(Side::Right);
    }

    fn frame_block(&self) -> Block<'static> {
        let (left_title, right_title) = match &self.mode {
            Mode::Solo { match_number, .. } => {
                (" You ".to_owned(), format!(" CPU · match {match_number} "))
            }
            Mode::Versus => (" P1 · w s ".to_owned(), " P2 · ↑ ↓ ".to_owned()),
        };
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray())
            .title_top(Line::from(left_title).light_cyan())
            .title_top(Line::from(right_title).light_magenta().right_aligned())
    }

    fn draw_court(&self, buffer: &mut Buffer, court: Rect) {
        let faint = Style::new().fg(Color::DarkGray);
        for row in 0..ROWS {
            buffer.set_string(court.x + NET_COLUMN, court.y + row, "┊", faint);
        }
        digits::draw(
            buffer,
            court.x + COLUMNS / 4,
            court.y + SCORE_TOP,
            self.left_points,
            faint,
        );
        digits::draw(
            buffer,
            court.x + COLUMNS * 3 / 4,
            court.y + SCORE_TOP,
            self.right_points,
            faint,
        );

        draw_paddle(buffer, court, self.left, Color::LightCyan);
        draw_paddle(buffer, court, self.right, Color::LightMagenta);

        let ball_style = Style::new().fg(Color::LightYellow);
        buffer.set_string(
            court.x + self.ball.x as u16,
            court.y + self.ball.y as u16,
            "●",
            ball_style,
        );
    }
}

impl Game for Pong {
    fn size(&self) -> Size {
        Size::new(COLUMNS + 2, ROWS + 2)
    }

    fn handle_key(&mut self, key: KeyCode, _speaker: &Speaker) {
        self.held.press(key);
    }

    fn handle_key_release(&mut self, key: KeyCode) {
        self.held.release(key);
    }

    fn update(&mut self, elapsed: Duration, speaker: &Speaker) {
        self.steer_paddles(elapsed);
        self.held.tick(elapsed);
        if !self.serve_in.is_zero() {
            self.serve_in = self.serve_in.saturating_sub(elapsed);
            return;
        }
        let scorer = self
            .ball
            .substeps(elapsed)
            .find_map(|step| self.step_ball(step, speaker));
        if let Some(scorer) = scorer {
            self.award_point(scorer, speaker);
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = self.frame_block();
        let court = block.inner(area);
        block.render(area, frame.buffer_mut());
        self.draw_court(frame.buffer_mut(), court);
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn is_over(&self) -> bool {
        self.winner.is_some()
    }

    fn is_won(&self) -> bool {
        matches!(self.mode, Mode::Versus) && self.winner.is_some()
    }

    fn summary(&self) -> Option<String> {
        let tally = format!("{}–{}", self.left_points, self.right_points);
        match (&self.mode, self.winner?) {
            (Mode::Solo { match_number, .. }, _) => {
                Some(format!("Lost match {match_number} · {tally}"))
            }
            (Mode::Versus, Side::Left) => Some(format!("P1 wins · {tally}")),
            (Mode::Versus, Side::Right) => Some(format!("P2 wins · {tally}")),
        }
    }
}

fn direction(held: &HeldKeys, controls: &Controls) -> f32 {
    match (held.is_held(controls.up), held.is_held(controls.down)) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

fn draw_paddle(buffer: &mut Buffer, court: Rect, paddle: Paddle, color: Color) {
    let top = paddle.top.round() as u16;
    for row in top..top + paddle::HEIGHT as u16 {
        buffer.set_string(
            court.x + paddle.column,
            court.y + row,
            "█",
            Style::new().fg(color),
        );
    }
}
