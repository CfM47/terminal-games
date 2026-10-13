mod field;
mod sounds;

use std::time::Duration;

use crossterm::event::{KeyCode, MouseButton};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Position, Rect, Size},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Widget},
};

use self::field::{Cell, Cover, Field, Reveal, Spot};
use crate::{audio::Speaker, game::Game};

const CELL_WIDTH: u16 = 2;
const CURSOR_BACKGROUND: Color = Color::Indexed(238);
const NUMBER_COLORS: [Color; 8] = [
    Color::LightBlue,
    Color::LightGreen,
    Color::LightRed,
    Color::Indexed(63),
    Color::Indexed(124),
    Color::Cyan,
    Color::White,
    Color::Gray,
];

pub struct Difficulty {
    name: &'static str,
    columns: usize,
    rows: usize,
    mines: usize,
}

pub const BEGINNER: Difficulty = Difficulty {
    name: "Beginner",
    columns: 9,
    rows: 9,
    mines: 10,
};

pub const INTERMEDIATE: Difficulty = Difficulty {
    name: "Intermediate",
    columns: 16,
    rows: 16,
    mines: 40,
};

pub const EXPERT: Difficulty = Difficulty {
    name: "Expert",
    columns: 30,
    rows: 16,
    mines: 99,
};

pub struct Minesweeper {
    difficulty: &'static Difficulty,
    field: Field,
    cursor: Spot,
    elapsed: Duration,
    started: bool,
    exploded: Option<Spot>,
    won: bool,
}

impl Minesweeper {
    pub fn new(difficulty: &'static Difficulty) -> Self {
        Self {
            difficulty,
            field: Field::new(difficulty.columns, difficulty.rows, difficulty.mines),
            cursor: Spot {
                column: difficulty.columns / 2,
                row: difficulty.rows / 2,
            },
            elapsed: Duration::ZERO,
            started: false,
            exploded: None,
            won: false,
        }
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let column = self.cursor.column.saturating_add_signed(dx);
        let row = self.cursor.row.saturating_add_signed(dy);
        self.cursor = Spot {
            column: column.min(self.difficulty.columns - 1),
            row: row.min(self.difficulty.rows - 1),
        };
    }

    fn dig(&mut self, spot: Spot, speaker: &Speaker) {
        let outcome = match self.field.cell(spot).cover {
            Cover::Open => self.field.chord(spot),
            Cover::Hidden | Cover::Flagged => self.field.reveal(spot),
        };
        self.started = true;
        match outcome {
            Reveal::Exploded(mine) => {
                self.exploded = Some(mine);
                speaker.play(&sounds::EXPLOSION);
            }
            Reveal::Opened(_) if self.field.is_cleared() => {
                self.won = true;
                self.field.flag_all_mines();
            }
            Reveal::Opened(1) => speaker.play(&sounds::OPEN),
            Reveal::Opened(_) => speaker.play(&sounds::OPEN_AREA),
            Reveal::Nothing => {}
        }
    }

    fn flag(&mut self, spot: Spot, speaker: &Speaker) {
        if self.field.toggle_flag(spot) {
            let flagged = self.field.cell(spot).cover == Cover::Flagged;
            speaker.play(if flagged {
                &sounds::FLAG
            } else {
                &sounds::UNFLAG
            });
        }
    }

    fn spot_at(&self, position: Position) -> Option<Spot> {
        let column = usize::from(position.x.checked_sub(1)? / CELL_WIDTH);
        let row = usize::from(position.y.checked_sub(1)?);
        (column < self.difficulty.columns && row < self.difficulty.rows)
            .then_some(Spot { column, row })
    }

    fn frame_block(&self) -> Block<'static> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray())
            .title_top(Line::from(format!(" ⚑ {} ", self.field.flags_left())).light_red())
            .title_top(Line::from(format!(" {} ", self.difficulty.name)).right_aligned())
    }

    fn draw_cell(&self, buffer: &mut Buffer, field: Rect, spot: Spot) {
        let (symbol, mut style) = self.appearance(spot, self.field.cell(spot));
        if spot == self.cursor && !self.is_over() {
            style = style.bg(CURSOR_BACKGROUND);
        }
        let x = field.x + spot.column as u16 * CELL_WIDTH;
        buffer.set_string(x, field.y + spot.row as u16, symbol, style);
    }

    fn appearance(&self, spot: Spot, cell: Cell) -> (String, Style) {
        let lost = self.exploded.is_some();
        let look = |symbol: &str, color: Color| (format!(" {symbol}"), Style::new().fg(color));
        match cell.cover {
            Cover::Hidden if lost && cell.mine => look("●", Color::White),
            Cover::Hidden => look("■", Color::Indexed(244)),
            Cover::Flagged if lost && !cell.mine => look("✗", Color::LightRed),
            Cover::Flagged => look("⚑", Color::LightRed),
            Cover::Open if cell.mine => {
                let (symbol, style) = look("●", Color::White);
                let exploded = self.exploded == Some(spot);
                (
                    symbol,
                    if exploded {
                        style.bg(Color::Red)
                    } else {
                        style
                    },
                )
            }
            Cover::Open if cell.adjacent_mines == 0 => look(" ", Color::Reset),
            Cover::Open => {
                let color = NUMBER_COLORS[usize::from(cell.adjacent_mines) - 1];
                let (symbol, style) = look(&cell.adjacent_mines.to_string(), color);
                (symbol, style.bold())
            }
        }
    }
}

impl Game for Minesweeper {
    fn size(&self) -> Size {
        let columns = self.difficulty.columns as u16;
        Size::new(columns * CELL_WIDTH + 3, self.difficulty.rows as u16 + 2)
    }

    fn handle_key(&mut self, key: KeyCode, speaker: &Speaker) {
        match key {
            KeyCode::Up | KeyCode::Char('w') => self.move_cursor(0, -1),
            KeyCode::Down | KeyCode::Char('s') => self.move_cursor(0, 1),
            KeyCode::Left | KeyCode::Char('a') => self.move_cursor(-1, 0),
            KeyCode::Right | KeyCode::Char('d') => self.move_cursor(1, 0),
            KeyCode::Char(' ') | KeyCode::Enter => self.dig(self.cursor, speaker),
            KeyCode::Char('f') => self.flag(self.cursor, speaker),
            _ => {}
        }
    }

    fn handle_click(&mut self, position: Position, button: MouseButton, speaker: &Speaker) {
        let Some(spot) = self.spot_at(position) else {
            return;
        };
        self.cursor = spot;
        match button {
            MouseButton::Left | MouseButton::Middle => self.dig(spot, speaker),
            MouseButton::Right => self.flag(spot, speaker),
        }
    }

    fn update(&mut self, elapsed: Duration, _speaker: &Speaker) {
        if self.started && !self.is_over() {
            self.elapsed += elapsed;
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = self.frame_block();
        let field = block.inner(area);
        let buffer = frame.buffer_mut();
        block.render(area, buffer);
        for spot in self.field.spots() {
            self.draw_cell(buffer, field, spot);
        }
    }

    fn score(&self) -> u32 {
        self.elapsed.as_millis() as u32
    }

    fn is_over(&self) -> bool {
        self.won || self.exploded.is_some()
    }

    fn record(&self) -> Option<u32> {
        self.won.then(|| self.score())
    }

    fn is_won(&self) -> bool {
        self.won
    }

    fn summary(&self) -> Option<String> {
        let verdict = if self.won { "Field cleared!" } else { "Boom!" };
        Some(verdict.to_owned())
    }
}
