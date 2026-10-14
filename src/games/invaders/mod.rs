mod formation;
mod shields;
mod sounds;

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
    formation::{Formation, SPRITE_WIDTH},
    shields::Shields,
};
use crate::{audio::Speaker, game::Game, input::HeldKeys};

const COLUMNS: u16 = 60;
const ROWS: u16 = 24;
const UFO_ROW: u16 = 0;
const FORMATION_TOP: u16 = 2;
const MAX_EXTRA_DEPTH: u16 = 4;
const SHIELD_TOP: u16 = ROWS - 6;
const CANNON_ROW: u16 = ROWS - 1;
const CANNON_WIDTH: u16 = 3;
const CANNON_SPEED: f32 = 30.0;
const SHOT_SPEED: f32 = 36.0;
const BOMB_SPEED: f32 = 14.0;
const MAX_BOMBS: usize = 3;
const STARTING_LIVES: u32 = 3;
const SLOWEST_MARCH: Duration = Duration::from_millis(550);
const FASTEST_MARCH: Duration = Duration::from_millis(40);
const MARCH_SPEEDUP_PER_WAVE: f32 = 0.88;
const BOMB_INTERVAL: Duration = Duration::from_millis(900);
const UFO_INTERVAL: Duration = Duration::from_secs(18);
const UFO_SPEED: f32 = 12.0;
const UFO_SIREN_INTERVAL: Duration = Duration::from_millis(240);
const UFO_POINTS: [u32; 4] = [50, 100, 150, 300];
const RESPAWN_DELAY: Duration = Duration::from_millis(1200);
const BURST_DURATION: Duration = Duration::from_millis(250);
const LEFT_KEYS: [KeyCode; 2] = [KeyCode::Left, KeyCode::Char('a')];
const RIGHT_KEYS: [KeyCode; 2] = [KeyCode::Right, KeyCode::Char('d')];

#[derive(Clone, Copy)]
struct Shot {
    x: u16,
    y: f32,
}

struct Ufo {
    x: f32,
    heading: f32,
    points: u32,
}

impl Ufo {
    fn covers(&self, x: u16, y: u16) -> bool {
        let left = self.x.round();
        y == UFO_ROW && (left..left + f32::from(SPRITE_WIDTH)).contains(&f32::from(x))
    }
}

struct Burst {
    x: u16,
    y: u16,
    label: String,
    remaining: Duration,
}

pub struct Invaders {
    formation: Formation,
    shields: Shields,
    cannon_left: f32,
    held: HeldKeys,
    shot: Option<Shot>,
    bombs: Vec<Shot>,
    ufo: Option<Ufo>,
    next_ufo_in: Duration,
    siren_in: Duration,
    march_in: Duration,
    march_note: usize,
    next_bomb_in: Duration,
    respawn_in: Option<Duration>,
    bursts: Vec<Burst>,
    lives: u32,
    wave: u32,
    score: u32,
    invaded: bool,
}

impl Invaders {
    pub fn new() -> Self {
        Self {
            formation: Formation::new(FORMATION_TOP),
            shields: Shields::new(COLUMNS, SHIELD_TOP),
            cannon_left: f32::from(COLUMNS - CANNON_WIDTH) / 2.0,
            held: HeldKeys::default(),
            shot: None,
            bombs: Vec::new(),
            ufo: None,
            next_ufo_in: UFO_INTERVAL,
            siren_in: Duration::ZERO,
            march_in: SLOWEST_MARCH,
            march_note: 0,
            next_bomb_in: BOMB_INTERVAL,
            respawn_in: None,
            bursts: Vec::new(),
            lives: STARTING_LIVES,
            wave: 1,
            score: 0,
            invaded: false,
        }
    }

    fn cannon_x(&self) -> u16 {
        self.cannon_left.round() as u16
    }

    fn cannon_alive(&self) -> bool {
        self.respawn_in.is_none()
    }

    fn fire(&mut self, speaker: &Speaker) {
        if self.shot.is_none() && self.cannon_alive() {
            let x = self.cannon_x() + CANNON_WIDTH / 2;
            self.shot = Some(Shot {
                x,
                y: f32::from(CANNON_ROW) - 1.0,
            });
            speaker.play(&sounds::SHOOT);
        }
    }

    fn steer_cannon(&mut self, elapsed: Duration) {
        let direction = match (
            self.held.is_held(&LEFT_KEYS),
            self.held.is_held(&RIGHT_KEYS),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let travel = direction * CANNON_SPEED * elapsed.as_secs_f32();
        self.cannon_left =
            (self.cannon_left + travel).clamp(0.0, f32::from(COLUMNS - CANNON_WIDTH));
    }

    fn march_interval(&self) -> Duration {
        let slowest = SLOWEST_MARCH.mul_f32(MARCH_SPEEDUP_PER_WAVE.powi(self.wave as i32 - 1));
        let share = self.formation.remaining() as f32 / Formation::SIZE as f32;
        FASTEST_MARCH + slowest.saturating_sub(FASTEST_MARCH).mul_f32(share)
    }

    fn march(&mut self, elapsed: Duration, speaker: &Speaker) {
        self.march_in = self.march_in.saturating_sub(elapsed);
        if !self.march_in.is_zero() {
            return;
        }
        self.formation.advance(COLUMNS);
        for invader in self.formation.invaders() {
            self.shields.erode(invader.x, invader.y, SPRITE_WIDTH);
        }
        speaker.play(&sounds::MARCH[self.march_note]);
        self.march_note = (self.march_note + 1) % sounds::MARCH.len();
        self.march_in = self.march_interval();
        self.invaded = self.formation.lowest_row() >= Some(CANNON_ROW);
    }

    fn drop_bombs(&mut self, elapsed: Duration) {
        self.next_bomb_in = self.next_bomb_in.saturating_sub(elapsed);
        if !self.next_bomb_in.is_zero() || self.bombs.len() >= MAX_BOMBS {
            return;
        }
        if let Some(shooter) = self.formation.random_shooter() {
            self.bombs.push(Shot {
                x: shooter.center(),
                y: f32::from(shooter.y + 1),
            });
        }
        let pace = 0.5 + fastrand::f32();
        self.next_bomb_in = BOMB_INTERVAL.mul_f32(pace / self.wave as f32);
    }

    fn fly_ufo(&mut self, elapsed: Duration, speaker: &Speaker) {
        let seconds = elapsed.as_secs_f32();
        match &mut self.ufo {
            Some(ufo) => {
                ufo.x += ufo.heading * UFO_SPEED * seconds;
                if !(-f32::from(SPRITE_WIDTH)..f32::from(COLUMNS)).contains(&ufo.x) {
                    self.ufo = None;
                    return;
                }
                self.siren_in = self.siren_in.saturating_sub(elapsed);
                if self.siren_in.is_zero() {
                    speaker.play(&sounds::UFO);
                    self.siren_in = UFO_SIREN_INTERVAL;
                }
            }
            None => {
                self.next_ufo_in = self.next_ufo_in.saturating_sub(elapsed);
                if self.next_ufo_in.is_zero() {
                    self.ufo = Some(launch_ufo());
                    self.next_ufo_in = UFO_INTERVAL.mul_f32(0.7 + fastrand::f32() * 0.6);
                }
            }
        }
    }

    fn move_shot(&mut self, elapsed: Duration, speaker: &Speaker) {
        let Some(shot) = self.shot else { return };
        let target = shot.y - SHOT_SPEED * elapsed.as_secs_f32();
        let hit = rows_crossed(shot.y, target)
            .into_iter()
            .any(|row| self.shot_lands(shot.x, row, speaker));
        self.shot = (!hit && target >= 0.0).then_some(Shot { y: target, ..shot });
    }

    fn shot_lands(&mut self, x: u16, row: u16, speaker: &Speaker) -> bool {
        if let Some(ufo) = self.ufo.take_if(|ufo| ufo.covers(x, row)) {
            self.score += ufo.points;
            self.burst(ufo.x.round().max(0.0) as u16, row, ufo.points.to_string());
            speaker.play(&sounds::UFO_HIT);
            return true;
        }
        if let Some(invader) = self.formation.strike(x, row) {
            self.score += invader.kind.points();
            self.burst(invader.x, invader.y, "\\*/".to_owned());
            speaker.play(&sounds::INVADER_DIES);
            return true;
        }
        self.shields.hit(x, row)
    }

    fn move_bombs(&mut self, elapsed: Duration, speaker: &Speaker) {
        let travel = BOMB_SPEED * elapsed.as_secs_f32();
        let bombs = std::mem::take(&mut self.bombs);
        for bomb in bombs {
            let target = bomb.y + travel;
            let landed = rows_crossed(bomb.y, target)
                .into_iter()
                .any(|row| self.bomb_lands(bomb.x, row, speaker));
            if !landed && target < f32::from(ROWS) {
                self.bombs.push(Shot { y: target, ..bomb });
            }
        }
    }

    fn bomb_lands(&mut self, x: u16, row: u16, speaker: &Speaker) -> bool {
        let cannon = self.cannon_x()..self.cannon_x() + CANNON_WIDTH;
        if row == CANNON_ROW && cannon.contains(&x) && self.cannon_alive() {
            self.destroy_cannon(speaker);
            return true;
        }
        self.shields.hit(x, row)
    }

    fn destroy_cannon(&mut self, speaker: &Speaker) {
        self.lives -= 1;
        self.respawn_in = Some(RESPAWN_DELAY);
        self.shot = None;
        self.bombs.clear();
        self.burst(self.cannon_x(), CANNON_ROW, "\\*/".to_owned());
        speaker.play(&sounds::CANNON_DIES);
    }

    fn burst(&mut self, x: u16, y: u16, label: String) {
        self.bursts.push(Burst {
            x,
            y,
            label,
            remaining: BURST_DURATION,
        });
    }

    fn start_next_wave(&mut self, speaker: &Speaker) {
        self.wave += 1;
        let depth = (self.wave as u16 - 1).min(MAX_EXTRA_DEPTH);
        self.formation = Formation::new(FORMATION_TOP + depth);
        self.shields = Shields::new(COLUMNS, SHIELD_TOP);
        self.shot = None;
        self.bombs.clear();
        self.march_in = SLOWEST_MARCH;
        speaker.play(&sounds::WAVE_CLEARED);
    }

    fn frame_block(&self) -> Block<'static> {
        let hearts = "♥ ".repeat(self.lives as usize);
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray())
            .title_top(Line::from(format!(" {hearts}")).light_red())
            .title_top(Line::from(format!(" Wave {} ", self.wave)).right_aligned())
    }

    fn draw_field(&self, buffer: &mut Buffer, field: Rect) {
        let mut draw = |x: u16, y: u16, symbol: &str, style: Style| {
            buffer.set_string(field.x + x, field.y + y, symbol, style);
        };
        for (x, y) in self.shields.blocks() {
            draw(x, y, "█", Style::new().fg(Color::Green));
        }
        for invader in self.formation.invaders() {
            let sprite = invader.kind.sprite(self.formation.legs_out);
            draw(
                invader.x,
                invader.y,
                sprite,
                Style::new().fg(invader.kind.color()).bold(),
            );
        }
        if let Some(ufo) = &self.ufo
            && ufo.x >= 0.0
        {
            draw(
                ufo.x.round() as u16,
                UFO_ROW,
                "<=>",
                Style::new().light_red().bold(),
            );
        }
        if let Some(shot) = self.shot {
            draw(shot.x, shot.y as u16, "│", Style::new().white());
        }
        for bomb in &self.bombs {
            draw(bomb.x, bomb.y as u16, "¦", Style::new().light_yellow());
        }
        if self.cannon_alive() {
            draw(
                self.cannon_x(),
                CANNON_ROW,
                "▄█▄",
                Style::new().light_green(),
            );
        }
        for burst in &self.bursts {
            draw(
                burst.x,
                burst.y,
                &burst.label,
                Style::new().light_yellow().bold(),
            );
        }
    }
}

impl Game for Invaders {
    fn size(&self) -> Size {
        Size::new(COLUMNS + 2, ROWS + 2)
    }

    fn handle_key(&mut self, key: KeyCode, speaker: &Speaker) {
        match key {
            KeyCode::Char(' ') | KeyCode::Up | KeyCode::Char('w') => self.fire(speaker),
            _ => self.held.press(key),
        }
    }

    fn handle_key_release(&mut self, key: KeyCode) {
        self.held.release(key);
    }

    fn update(&mut self, elapsed: Duration, speaker: &Speaker) {
        self.bursts.retain_mut(|burst| {
            burst.remaining = burst.remaining.saturating_sub(elapsed);
            !burst.remaining.is_zero()
        });
        if let Some(remaining) = self.respawn_in {
            let remaining = remaining.saturating_sub(elapsed);
            self.respawn_in = (!remaining.is_zero()).then_some(remaining);
            if self.respawn_in.is_none() {
                self.cannon_left = f32::from(COLUMNS - CANNON_WIDTH) / 2.0;
            }
            return;
        }
        self.steer_cannon(elapsed);
        self.held.tick(elapsed);
        self.march(elapsed, speaker);
        self.drop_bombs(elapsed);
        self.fly_ufo(elapsed, speaker);
        self.move_shot(elapsed, speaker);
        self.move_bombs(elapsed, speaker);
        if self.formation.remaining() == 0 {
            self.start_next_wave(speaker);
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
        self.lives == 0 || self.invaded
    }

    fn summary(&self) -> Option<String> {
        let reason = if self.invaded {
            "The invaders landed"
        } else {
            "Out of cannons"
        };
        Some(format!("{reason} on wave {}", self.wave))
    }
}

fn launch_ufo() -> Ufo {
    let from_left = fastrand::bool();
    Ufo {
        x: if from_left {
            -f32::from(SPRITE_WIDTH) + 0.01
        } else {
            f32::from(COLUMNS - 1)
        },
        heading: if from_left { 1.0 } else { -1.0 },
        points: fastrand::choice(UFO_POINTS).unwrap_or(UFO_POINTS[0]),
    }
}

/// Grid rows a projectile passes through moving between two heights, in travel order.
fn rows_crossed(from: f32, to: f32) -> Vec<u16> {
    let (start, end) = (from.max(0.0) as u16, to.max(0.0) as u16);
    if start <= end {
        (start..=end).collect()
    } else {
        (end..=start).rev().collect()
    }
}
