mod grid;
mod sounds;

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Widget},
};

use self::grid::{Direction, Grid, SIZE};
use crate::{audio::Speaker, game::Game};

const GOAL: u32 = 2048;
const TILE_WIDTH: u16 = 8;
const TILE_HEIGHT: u16 = 3;
const GAP_X: u16 = 2;
const GAP_Y: u16 = 1;
const TILES: u16 = SIZE as u16;
const GRID_WIDTH: u16 = TILES * TILE_WIDTH + (TILES - 1) * GAP_X;
const GRID_HEIGHT: u16 = TILES * TILE_HEIGHT + (TILES - 1) * GAP_Y;
const EMPTY_TILE: Color = Color::Indexed(237);
const DARK_TEXT: Color = Color::Indexed(235);
const LIGHT_TEXT: Color = Color::Indexed(255);
const TILE_COLORS: [(Color, Color); 11] = [
    (Color::Indexed(254), DARK_TEXT),
    (Color::Indexed(230), DARK_TEXT),
    (Color::Indexed(215), LIGHT_TEXT),
    (Color::Indexed(209), LIGHT_TEXT),
    (Color::Indexed(203), LIGHT_TEXT),
    (Color::Indexed(196), LIGHT_TEXT),
    (Color::Indexed(229), DARK_TEXT),
    (Color::Indexed(228), DARK_TEXT),
    (Color::Indexed(227), DARK_TEXT),
    (Color::Indexed(221), DARK_TEXT),
    (Color::Indexed(220), DARK_TEXT),
];
const BEYOND_GOAL_COLORS: (Color, Color) = (Color::Indexed(93), LIGHT_TEXT);

pub struct Game2048 {
    grid: Grid,
    score: u32,
    reached_goal: bool,
}

impl Game2048 {
    pub fn new() -> Self {
        let mut grid = Grid::new();
        grid.spawn_random();
        grid.spawn_random();
        Self {
            grid,
            score: 0,
            reached_goal: false,
        }
    }

    fn play(&mut self, direction: Direction, speaker: &Speaker) {
        let Some(points) = self.grid.slide(direction) else {
            return;
        };
        self.score += points;
        self.grid.spawn_random();
        let just_reached_goal = !self.reached_goal && self.grid.highest() >= GOAL;
        self.reached_goal |= just_reached_goal;
        speaker.play(match (just_reached_goal, points) {
            (true, _) => &sounds::GOAL,
            (false, 0) => &sounds::SLIDE,
            (false, _) => &sounds::MERGE,
        });
    }

    fn frame_block(&self) -> Block<'static> {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray())
            .padding(Padding::horizontal(1));
        if self.reached_goal {
            block.title_top(
                Line::from(" 2048 reached! ")
                    .light_yellow()
                    .bold()
                    .centered(),
            )
        } else {
            block
        }
    }
}

impl Game for Game2048 {
    fn size(&self) -> Size {
        Size::new(GRID_WIDTH + 4, GRID_HEIGHT + 2)
    }

    fn handle_key(&mut self, key: KeyCode, speaker: &Speaker) {
        let direction = match key {
            KeyCode::Up | KeyCode::Char('w') => Direction::Up,
            KeyCode::Down | KeyCode::Char('s') => Direction::Down,
            KeyCode::Left | KeyCode::Char('a') => Direction::Left,
            KeyCode::Right | KeyCode::Char('d') => Direction::Right,
            _ => return,
        };
        self.play(direction, speaker);
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = self.frame_block();
        let inner = block.inner(area);
        let buffer = frame.buffer_mut();
        block.render(area, buffer);
        for ((row, column), value) in self.grid.tiles() {
            draw_tile(buffer, tile_area(inner, row as u16, column as u16), value);
        }
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn is_over(&self) -> bool {
        !self.grid.can_move()
    }

    fn summary(&self) -> Option<String> {
        Some(format!("Highest tile {}", self.grid.highest()))
    }
}

fn tile_area(grid: Rect, row: u16, column: u16) -> Rect {
    Rect::new(
        grid.x + column * (TILE_WIDTH + GAP_X),
        grid.y + row * (TILE_HEIGHT + GAP_Y),
        TILE_WIDTH,
        TILE_HEIGHT,
    )
}

fn draw_tile(buffer: &mut Buffer, area: Rect, value: u32) {
    if value == 0 {
        buffer.set_style(area, Style::new().bg(EMPTY_TILE));
        return;
    }
    let (background, text) = tile_colors(value);
    let style = Style::new().bg(background).fg(text).bold();
    buffer.set_style(area, style);
    let label = value.to_string();
    let x = area.x + (area.width - label.len() as u16) / 2;
    buffer.set_string(x, area.y + area.height / 2, label, style);
}

fn tile_colors(value: u32) -> (Color, Color) {
    let tier = value.trailing_zeros() as usize - 1;
    TILE_COLORS.get(tier).copied().unwrap_or(BEYOND_GOAL_COLORS)
}
