mod board;
mod piece;
mod sounds;

use std::time::Duration;

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
};

use self::{
    board::{Board, HEIGHT, WIDTH},
    piece::{Bag, Kind, Piece, Spin},
};
use crate::{audio::Speaker, game::Game};

pub use sounds::KOROBEINIKI as MUSIC;

const CELL_WIDTH: u16 = 2;
const WELL_WIDTH: u16 = WIDTH as u16 * CELL_WIDTH + 2;
const WELL_HEIGHT: u16 = HEIGHT as u16 + 2;
const SIDEBAR_WIDTH: u16 = 12;
const PREVIEW_HEIGHT: u16 = 4;
const LOCK_DELAY: Duration = Duration::from_millis(500);
const MAX_LOCK_RESETS: u8 = 15;
const LINES_PER_LEVEL: u32 = 10;
const MUSIC_SPEEDUP_PER_LEVEL: f32 = 0.06;
const MAX_MUSIC_TEMPO: f32 = 2.0;
const LINE_SCORES: [u32; 5] = [0, 100, 300, 500, 800];
const WALL_KICKS: [(i16, i16); 6] = [(0, 0), (-1, 0), (1, 0), (0, -1), (-2, 0), (2, 0)];

pub struct Tetris {
    board: Board,
    bag: Bag,
    active: Piece,
    next: Kind,
    since_fall: Duration,
    grounded_for: Duration,
    lock_resets: u8,
    lines: u32,
    score: u32,
    topped_out: bool,
}

impl Tetris {
    pub fn new() -> Self {
        let mut bag = Bag::new();
        let active = spawn(bag.draw());
        let next = bag.draw();
        Self {
            board: Board::new(),
            bag,
            active,
            next,
            since_fall: Duration::ZERO,
            grounded_for: Duration::ZERO,
            lock_resets: 0,
            lines: 0,
            score: 0,
            topped_out: false,
        }
    }

    fn level(&self) -> u32 {
        self.lines / LINES_PER_LEVEL + 1
    }

    /// Gravity curve from the Tetris guideline.
    fn fall_interval(&self) -> Duration {
        let steps = f64::from(self.level() - 1);
        Duration::from_secs_f64((0.8 - steps * 0.007).max(0.0).powf(steps))
    }

    fn is_grounded(&self) -> bool {
        !self.board.fits(self.active.shifted(0, 1))
    }

    fn ghost(&self) -> Piece {
        self.active
            .shifted(0, self.board.drop_distance(self.active))
    }

    fn try_place(&mut self, candidate: Piece) -> bool {
        if !self.board.fits(candidate) {
            return false;
        }
        self.active = candidate;
        if self.grounded_for > Duration::ZERO && self.lock_resets < MAX_LOCK_RESETS {
            self.grounded_for = Duration::ZERO;
            self.lock_resets += 1;
        }
        true
    }

    fn shift(&mut self, dx: i16) {
        self.try_place(self.active.shifted(dx, 0));
    }

    fn spin(&mut self, spin: Spin, speaker: &Speaker) {
        let spun = self.active.spun(spin);
        let kicked = WALL_KICKS
            .iter()
            .map(|&(dx, dy)| spun.shifted(dx, dy))
            .find(|&candidate| self.board.fits(candidate));
        if let Some(candidate) = kicked {
            self.try_place(candidate);
            speaker.play(&sounds::ROTATE);
        }
    }

    fn soft_drop(&mut self) {
        if self.try_place(self.active.shifted(0, 1)) {
            self.score += 1;
            self.since_fall = Duration::ZERO;
        }
    }

    fn hard_drop(&mut self, speaker: &Speaker) {
        let distance = self.board.drop_distance(self.active);
        self.active = self.active.shifted(0, distance);
        self.score += 2 * distance as u32;
        self.lock(speaker);
    }

    fn fall(&mut self, elapsed: Duration) {
        self.grounded_for = Duration::ZERO;
        self.since_fall += elapsed;
        while self.since_fall >= self.fall_interval() && !self.is_grounded() {
            self.since_fall -= self.fall_interval();
            self.active = self.active.shifted(0, 1);
        }
    }

    fn rest(&mut self, elapsed: Duration, speaker: &Speaker) {
        self.since_fall = Duration::ZERO;
        self.grounded_for += elapsed;
        if self.grounded_for >= LOCK_DELAY {
            self.lock(speaker);
        }
    }

    fn lock(&mut self, speaker: &Speaker) {
        self.board.lock(self.active);
        let cleared = self.board.clear_full_rows();
        self.score += LINE_SCORES[cleared as usize] * self.level();
        self.lines += cleared;
        speaker.play(match cleared {
            0 => &sounds::LOCK,
            4 => &sounds::FOUR_LINES,
            _ => &sounds::LINE_CLEAR,
        });
        self.spawn_next();
    }

    fn spawn_next(&mut self) {
        self.active = spawn(self.next);
        self.next = self.bag.draw();
        self.since_fall = Duration::ZERO;
        self.grounded_for = Duration::ZERO;
        self.lock_resets = 0;
        self.topped_out = !self.board.fits(self.active);
    }

    fn render_well(&self, buffer: &mut Buffer, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray());
        let field = block.inner(area);
        block.render(area, buffer);

        let grid = (0..HEIGHT).flat_map(|y| (0..WIDTH).map(move |x| (x, y)));
        for cell in grid {
            paint(buffer, field, cell, " ·", Style::new().fg(Color::DarkGray));
        }
        for (cell, kind) in self.board.filled_cells() {
            paint(buffer, field, cell, "██", Style::new().fg(kind.color()));
        }
        let ghost_style = Style::new().fg(self.active.kind.color()).dim();
        for cell in self.ghost().cells() {
            paint(buffer, field, cell, "░░", ghost_style);
        }
        for cell in self.active.cells() {
            paint(
                buffer,
                field,
                cell,
                "██",
                Style::new().fg(self.active.kind.color()),
            );
        }
    }

    fn render_sidebar(&self, frame: &mut Frame, area: Rect) {
        let [preview, _, stats] = area.layout(&Layout::vertical([
            Constraint::Length(PREVIEW_HEIGHT),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]));
        render_preview(frame.buffer_mut(), preview, self.next);
        frame.render_widget(self.stats(), stats);
    }

    fn stats(&self) -> Paragraph<'static> {
        Paragraph::new(vec![
            Line::from("Level".dark_gray()),
            Line::from(self.level().to_string().bold()),
            Line::default(),
            Line::from("Lines".dark_gray()),
            Line::from(self.lines.to_string().bold()),
        ])
    }
}

impl Game for Tetris {
    fn size(&self) -> Size {
        Size::new(WELL_WIDTH + 1 + SIDEBAR_WIDTH, WELL_HEIGHT)
    }

    fn handle_key(&mut self, key: KeyCode, speaker: &Speaker) {
        match key {
            KeyCode::Left | KeyCode::Char('a') => self.shift(-1),
            KeyCode::Right | KeyCode::Char('d') => self.shift(1),
            KeyCode::Down | KeyCode::Char('s') => self.soft_drop(),
            KeyCode::Up | KeyCode::Char('w' | 'x') => self.spin(Spin::Clockwise, speaker),
            KeyCode::Char('z') => self.spin(Spin::CounterClockwise, speaker),
            KeyCode::Char(' ') => self.hard_drop(speaker),
            _ => {}
        }
    }

    fn update(&mut self, elapsed: Duration, speaker: &Speaker) {
        if self.is_grounded() {
            self.rest(elapsed, speaker);
        } else {
            self.fall(elapsed);
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let [well, _, sidebar] = area.layout(&Layout::horizontal([
            Constraint::Length(WELL_WIDTH),
            Constraint::Length(1),
            Constraint::Length(SIDEBAR_WIDTH),
        ]));
        self.render_well(frame.buffer_mut(), well);
        self.render_sidebar(frame, sidebar);
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn is_over(&self) -> bool {
        self.topped_out
    }

    fn music_tempo(&self) -> f32 {
        (1.0 + MUSIC_SPEEDUP_PER_LEVEL * (self.level() - 1) as f32).min(MAX_MUSIC_TEMPO)
    }
}

fn spawn(kind: Kind) -> Piece {
    Piece::new(kind, (WIDTH - kind.box_size()) / 2, 0)
}

fn render_preview(buffer: &mut Buffer, area: Rect, kind: Kind) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dark_gray())
        .title(" Next ");
    let inner = block.inner(area);
    block.render(area, buffer);

    let piece = Piece::new(kind, 0, 0);
    let top = piece.cells().map(|(_, y)| y).min().unwrap_or(0);
    let left = (inner.width / CELL_WIDTH).saturating_sub(kind.box_size() as u16) as i16 / 2;
    for cell in piece.shifted(left, -top).cells() {
        paint(buffer, inner, cell, "██", Style::new().fg(kind.color()));
    }
}

fn paint(buffer: &mut Buffer, origin: Rect, (x, y): (i16, i16), symbol: &str, style: Style) {
    let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
        return;
    };
    buffer.set_string(origin.x + x * CELL_WIDTH, origin.y + y, symbol, style);
}
