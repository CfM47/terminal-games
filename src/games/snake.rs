use std::{collections::VecDeque, time::Duration};

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Style},
    widgets::{Block, BorderType},
};

use crate::{
    audio::{Duty, Note, Speaker, Tune, pitch},
    game::Game,
};

const COLUMNS: u16 = 24;
const ROWS: u16 = 16;
const CELL_WIDTH: u16 = 2;
const STARTING_LENGTH: u16 = 3;
const POINTS_PER_FOOD: u32 = 10;
const MAX_QUEUED_TURNS: usize = 2;
const START_INTERVAL: Duration = Duration::from_millis(150);
const FASTEST_INTERVAL: Duration = Duration::from_millis(60);
const SPEEDUP_PER_FOOD: Duration = Duration::from_millis(4);

const EAT: Tune = Tune {
    duty: Duty::Quarter,
    volume: 0.1,
    notes: &[Note::tone(pitch::B5, 50), Note::tone(pitch::E6, 110)],
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn from_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Up | KeyCode::Char('w') => Some(Self::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(Self::Down),
            KeyCode::Left | KeyCode::Char('a') => Some(Self::Left),
            KeyCode::Right | KeyCode::Char('d') => Some(Self::Right),
            _ => None,
        }
    }

    fn offset(self) -> (i16, i16) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    x: u16,
    y: u16,
}

impl Cell {
    fn neighbor(self, direction: Direction) -> Option<Self> {
        let (dx, dy) = direction.offset();
        let x = self.x.checked_add_signed(dx)?;
        let y = self.y.checked_add_signed(dy)?;
        (x < COLUMNS && y < ROWS).then_some(Self { x, y })
    }

    fn all() -> impl Iterator<Item = Self> {
        (0..ROWS).flat_map(|y| (0..COLUMNS).map(move |x| Self { x, y }))
    }
}

pub struct Snake {
    body: VecDeque<Cell>,
    heading: Direction,
    queued_turns: VecDeque<Direction>,
    food: Option<Cell>,
    since_last_step: Duration,
    eaten: u32,
    crashed: bool,
}

impl Snake {
    pub fn new() -> Self {
        let tail = Cell {
            x: COLUMNS / 4,
            y: ROWS / 2,
        };
        let body = (0..STARTING_LENGTH)
            .rev()
            .map(|offset| Cell {
                x: tail.x + offset,
                y: tail.y,
            })
            .collect();
        let mut snake = Self {
            body,
            heading: Direction::Right,
            queued_turns: VecDeque::new(),
            food: None,
            since_last_step: Duration::ZERO,
            eaten: 0,
            crashed: false,
        };
        snake.food = snake.random_free_cell();
        snake
    }

    fn head(&self) -> Cell {
        self.body[0]
    }

    fn step_interval(&self) -> Duration {
        START_INTERVAL
            .saturating_sub(SPEEDUP_PER_FOOD * self.eaten)
            .max(FASTEST_INTERVAL)
    }

    fn queue_turn(&mut self, direction: Direction) {
        let latest = self.queued_turns.back().copied().unwrap_or(self.heading);
        let is_real_turn = direction != latest && direction != latest.opposite();
        if is_real_turn && self.queued_turns.len() < MAX_QUEUED_TURNS {
            self.queued_turns.push_back(direction);
        }
    }

    fn step(&mut self, speaker: &Speaker) {
        if let Some(turn) = self.queued_turns.pop_front() {
            self.heading = turn;
        }
        let Some(next) = self.head().neighbor(self.heading) else {
            self.crashed = true;
            return;
        };
        let eats = self.food == Some(next);
        if !eats {
            self.body.pop_back();
        }
        if self.body.contains(&next) {
            self.crashed = true;
            return;
        }
        self.body.push_front(next);
        if eats {
            self.eaten += 1;
            self.food = self.random_free_cell();
            speaker.play(&EAT);
        }
    }

    fn random_free_cell(&self) -> Option<Cell> {
        let free: Vec<Cell> = Cell::all()
            .filter(|cell| !self.body.contains(cell))
            .collect();
        fastrand::choice(free)
    }

    fn head_style(&self) -> Style {
        let color = if self.crashed {
            Color::Red
        } else {
            Color::LightGreen
        };
        Style::new().fg(color)
    }
}

impl Game for Snake {
    fn size(&self) -> Size {
        Size::new(COLUMNS * CELL_WIDTH + 2, ROWS + 2)
    }

    fn handle_key(&mut self, key: KeyCode, _speaker: &Speaker) {
        if let Some(direction) = Direction::from_key(key) {
            self.queue_turn(direction);
        }
    }

    fn update(&mut self, elapsed: Duration, speaker: &Speaker) {
        self.since_last_step += elapsed;
        while !self.is_over() && self.since_last_step >= self.step_interval() {
            self.since_last_step -= self.step_interval();
            self.step(speaker);
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Green));
        let field = block.inner(area);
        frame.render_widget(block, area);

        let buffer = frame.buffer_mut();
        if let Some(food) = self.food {
            paint(buffer, field, food, "▐▌", Style::new().fg(Color::LightRed));
        }
        for &segment in self.body.iter().skip(1) {
            paint(buffer, field, segment, "██", Style::new().fg(Color::Green));
        }
        paint(buffer, field, self.head(), "██", self.head_style());
    }

    fn score(&self) -> u32 {
        self.eaten * POINTS_PER_FOOD
    }

    fn is_over(&self) -> bool {
        self.crashed || self.food.is_none()
    }
}

fn paint(buffer: &mut Buffer, field: Rect, cell: Cell, symbol: &str, style: Style) {
    buffer.set_string(
        field.x + cell.x * CELL_WIDTH,
        field.y + cell.y,
        symbol,
        style,
    );
}
