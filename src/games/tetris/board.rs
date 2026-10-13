use super::piece::{Kind, Piece};

pub const WIDTH: i16 = 10;
pub const HEIGHT: i16 = 20;

type Row = [Option<Kind>; WIDTH as usize];

const EMPTY_ROW: Row = [None; WIDTH as usize];

pub struct Board {
    rows: Vec<Row>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            rows: vec![EMPTY_ROW; HEIGHT as usize],
        }
    }

    pub fn fits(&self, piece: Piece) -> bool {
        piece.cells().all(|(x, y)| self.is_free(x, y))
    }

    /// Rows the piece can fall before it would collide.
    pub fn drop_distance(&self, piece: Piece) -> i16 {
        (0..)
            .take_while(|&distance| self.fits(piece.shifted(0, distance + 1)))
            .count() as i16
    }

    pub fn lock(&mut self, piece: Piece) {
        for (x, y) in piece.cells() {
            if let Some(slot) = self.slot_mut(x, y) {
                *slot = Some(piece.kind);
            }
        }
    }

    /// Removes completed rows and returns how many there were.
    pub fn clear_full_rows(&mut self) -> u32 {
        self.rows.retain(|row| row.iter().any(Option::is_none));
        let cleared = HEIGHT as usize - self.rows.len();
        self.rows.splice(0..0, [EMPTY_ROW].repeat(cleared));
        cleared as u32
    }

    pub fn filled_cells(&self) -> impl Iterator<Item = ((i16, i16), Kind)> {
        self.rows.iter().zip(0..).flat_map(|(row, y)| {
            row.iter()
                .zip(0..)
                .filter_map(move |(slot, x)| slot.map(|kind| ((x, y), kind)))
        })
    }

    fn is_free(&self, x: i16, y: i16) -> bool {
        (0..WIDTH).contains(&x)
            && y < HEIGHT
            && (y < 0 || self.rows[y as usize][x as usize].is_none())
    }

    fn slot_mut(&mut self, x: i16, y: i16) -> Option<&mut Option<Kind>> {
        let row = self.rows.get_mut(usize::try_from(y).ok()?)?;
        row.get_mut(usize::try_from(x).ok()?)
    }
}
