use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Kind {
    const ALL: [Kind; 7] = [
        Kind::I,
        Kind::O,
        Kind::T,
        Kind::S,
        Kind::Z,
        Kind::J,
        Kind::L,
    ];

    pub fn color(self) -> Color {
        match self {
            Kind::I => Color::LightCyan,
            Kind::O => Color::LightYellow,
            Kind::T => Color::LightMagenta,
            Kind::S => Color::LightGreen,
            Kind::Z => Color::LightRed,
            Kind::J => Color::LightBlue,
            Kind::L => Color::Indexed(208),
        }
    }

    /// Cells in spawn orientation inside the piece's square bounding box.
    fn shape(self) -> [(i16, i16); 4] {
        match self {
            Kind::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            Kind::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Kind::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            Kind::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            Kind::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
            Kind::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            Kind::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        }
    }

    pub fn box_size(self) -> i16 {
        match self {
            Kind::I => 4,
            Kind::O => 2,
            _ => 3,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Spin {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Copy)]
pub struct Piece {
    pub kind: Kind,
    quarter_turns: u8,
    x: i16,
    y: i16,
}

impl Piece {
    pub fn new(kind: Kind, x: i16, y: i16) -> Self {
        Self {
            kind,
            quarter_turns: 0,
            x,
            y,
        }
    }

    pub fn cells(self) -> impl Iterator<Item = (i16, i16)> {
        let last = self.kind.box_size() - 1;
        self.kind.shape().into_iter().map(move |cell| {
            let (x, y) = (0..self.quarter_turns).fold(cell, |(x, y), _| (last - y, x));
            (self.x + x, self.y + y)
        })
    }

    pub fn shifted(self, dx: i16, dy: i16) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }

    pub fn spun(self, spin: Spin) -> Self {
        let quarter_turns = match spin {
            Spin::Clockwise => (self.quarter_turns + 1) % 4,
            Spin::CounterClockwise => (self.quarter_turns + 3) % 4,
        };
        Self {
            quarter_turns,
            ..self
        }
    }
}

/// Seven-bag randomizer: every piece appears once before any repeats.
pub struct Bag {
    pending: Vec<Kind>,
}

impl Bag {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn draw(&mut self) -> Kind {
        if self.pending.is_empty() {
            self.pending = Kind::ALL.to_vec();
            fastrand::shuffle(&mut self.pending);
        }
        self.pending.pop().unwrap()
    }
}
