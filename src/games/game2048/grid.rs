pub const SIZE: usize = 4;
const FOUR_CHANCE: f32 = 0.1;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
}

#[derive(Clone)]
pub struct Grid {
    cells: [[u32; SIZE]; SIZE],
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: [[0; SIZE]; SIZE],
        }
    }

    pub fn spawn_random(&mut self) {
        let empty: Vec<(usize, usize)> = self
            .tiles()
            .filter(|&(_, value)| value == 0)
            .map(|(position, _)| position)
            .collect();
        if let Some((row, column)) = fastrand::choice(empty) {
            self.cells[row][column] = if fastrand::f32() < FOUR_CHANCE { 4 } else { 2 };
        }
    }

    /// Slides every tile toward one side, returning the merge points or `None` when nothing moved.
    pub fn slide(&mut self, direction: Direction) -> Option<u32> {
        let mut moved = false;
        let mut points = 0;
        for index in 0..SIZE {
            let positions = line(direction, index);
            let before = positions.map(|(row, column)| self.cells[row][column]);
            let (after, gained) = merge(before);
            for ((row, column), value) in positions.into_iter().zip(after) {
                self.cells[row][column] = value;
            }
            moved |= before != after;
            points += gained;
        }
        moved.then_some(points)
    }

    pub fn can_move(&self) -> bool {
        Direction::ALL
            .into_iter()
            .any(|direction| self.clone().slide(direction).is_some())
    }

    pub fn highest(&self) -> u32 {
        self.cells.iter().flatten().copied().max().unwrap_or(0)
    }

    pub fn tiles(&self) -> impl Iterator<Item = ((usize, usize), u32)> {
        self.cells.iter().enumerate().flat_map(|(row, values)| {
            values
                .iter()
                .enumerate()
                .map(move |(column, &value)| ((row, column), value))
        })
    }
}

/// Cells of one row or column, ordered from the edge the tiles slide toward.
fn line(direction: Direction, index: usize) -> [(usize, usize); SIZE] {
    std::array::from_fn(|step| match direction {
        Direction::Left => (index, step),
        Direction::Right => (index, SIZE - 1 - step),
        Direction::Up => (step, index),
        Direction::Down => (SIZE - 1 - step, index),
    })
}

/// Packs a line toward its start, merging each equal pair once, and returns the points earned.
fn merge(values: [u32; SIZE]) -> ([u32; SIZE], u32) {
    let mut packed = Vec::with_capacity(SIZE);
    let mut points = 0;
    let mut pending = None;
    for value in values.into_iter().filter(|&value| value != 0) {
        match pending.take() {
            Some(previous) if previous == value => {
                packed.push(value * 2);
                points += value * 2;
            }
            Some(previous) => {
                packed.push(previous);
                pending = Some(value);
            }
            None => pending = Some(value),
        }
    }
    packed.extend(pending);
    packed.resize(SIZE, 0);
    (packed.try_into().unwrap(), points)
}
