#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Spot {
    pub column: usize,
    pub row: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cover {
    Hidden,
    Flagged,
    Open,
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub mine: bool,
    pub adjacent_mines: u8,
    pub cover: Cover,
}

pub enum Reveal {
    Nothing,
    Opened(usize),
    Exploded(Spot),
}

pub struct Field {
    columns: usize,
    rows: usize,
    mines: usize,
    cells: Vec<Cell>,
    mines_laid: bool,
}

impl Field {
    pub fn new(columns: usize, rows: usize, mines: usize) -> Self {
        let blank = Cell {
            mine: false,
            adjacent_mines: 0,
            cover: Cover::Hidden,
        };
        Self {
            columns,
            rows,
            mines,
            cells: vec![blank; columns * rows],
            mines_laid: false,
        }
    }

    pub fn cell(&self, spot: Spot) -> Cell {
        self.cells[self.index(spot)]
    }

    pub fn flags_left(&self) -> isize {
        let flags = self
            .cells
            .iter()
            .filter(|cell| cell.cover == Cover::Flagged)
            .count();
        self.mines as isize - flags as isize
    }

    pub fn is_cleared(&self) -> bool {
        self.cells
            .iter()
            .all(|cell| cell.mine || cell.cover == Cover::Open)
    }

    pub fn toggle_flag(&mut self, spot: Spot) -> bool {
        let index = self.index(spot);
        let cover = &mut self.cells[index].cover;
        *cover = match *cover {
            Cover::Hidden => Cover::Flagged,
            Cover::Flagged => Cover::Hidden,
            Cover::Open => return false,
        };
        true
    }

    pub fn flag_all_mines(&mut self) {
        for cell in self.cells.iter_mut().filter(|cell| cell.mine) {
            cell.cover = Cover::Flagged;
        }
    }

    /// Opens a cell, laying the mines on the first reveal so it and its neighbors are always safe.
    pub fn reveal(&mut self, spot: Spot) -> Reveal {
        if !self.mines_laid {
            self.lay_mines(spot);
        }
        let cell = self.cell(spot);
        match (cell.cover, cell.mine) {
            (Cover::Hidden, true) => {
                let index = self.index(spot);
                self.cells[index].cover = Cover::Open;
                Reveal::Exploded(spot)
            }
            (Cover::Hidden, false) => Reveal::Opened(self.flood_open(spot)),
            _ => Reveal::Nothing,
        }
    }

    /// Opens every unflagged neighbor of a number whose mines are all flagged.
    pub fn chord(&mut self, spot: Spot) -> Reveal {
        let cell = self.cell(spot);
        let flagged = self
            .neighbors(spot)
            .filter(|&neighbor| self.cell(neighbor).cover == Cover::Flagged)
            .count();
        if cell.cover != Cover::Open || flagged != usize::from(cell.adjacent_mines) {
            return Reveal::Nothing;
        }
        let neighbors: Vec<Spot> = self.neighbors(spot).collect();
        let mut opened = 0;
        for neighbor in neighbors {
            match self.reveal(neighbor) {
                Reveal::Exploded(spot) => return Reveal::Exploded(spot),
                Reveal::Opened(count) => opened += count,
                Reveal::Nothing => {}
            }
        }
        if opened == 0 {
            Reveal::Nothing
        } else {
            Reveal::Opened(opened)
        }
    }

    pub fn spots(&self) -> impl Iterator<Item = Spot> + use<> {
        let columns = self.columns;
        (0..self.rows).flat_map(move |row| (0..columns).map(move |column| Spot { column, row }))
    }

    pub fn neighbors(&self, spot: Spot) -> impl Iterator<Item = Spot> + use<> {
        let (columns, rows) = (self.columns, self.rows);
        (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
            .filter(|&offset| offset != (0, 0))
            .filter_map(move |(dx, dy)| {
                let column = spot.column.checked_add_signed(dx)?;
                let row = spot.row.checked_add_signed(dy)?;
                (column < columns && row < rows).then_some(Spot { column, row })
            })
    }

    fn lay_mines(&mut self, safe: Spot) {
        let keep_clear: Vec<Spot> = self.neighbors(safe).chain([safe]).collect();
        let mut candidates: Vec<Spot> = self
            .spots()
            .filter(|spot| !keep_clear.contains(spot))
            .collect();
        fastrand::shuffle(&mut candidates);
        for spot in candidates.into_iter().take(self.mines) {
            let index = self.index(spot);
            self.cells[index].mine = true;
            for neighbor in self.neighbors(spot) {
                let index = self.index(neighbor);
                self.cells[index].adjacent_mines += 1;
            }
        }
        self.mines_laid = true;
    }

    fn flood_open(&mut self, start: Spot) -> usize {
        let mut pending = vec![start];
        let mut opened = 0;
        while let Some(spot) = pending.pop() {
            let index = self.index(spot);
            let cell = &mut self.cells[index];
            if cell.cover != Cover::Hidden || cell.mine {
                continue;
            }
            cell.cover = Cover::Open;
            opened += 1;
            if cell.adjacent_mines == 0 {
                pending.extend(self.neighbors(spot));
            }
        }
        opened
    }

    fn index(&self, spot: Spot) -> usize {
        spot.row * self.columns + spot.column
    }
}
