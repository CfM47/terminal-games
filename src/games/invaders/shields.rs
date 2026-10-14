use std::collections::HashSet;

const SHAPE: [&str; 3] = [" #### ", "######", "##  ##"];
const COUNT: u16 = 4;
const WIDTH: u16 = SHAPE[0].len() as u16;

/// Bunkers that crumble one block at a time.
pub struct Shields {
    blocks: HashSet<(u16, u16)>,
}

impl Shields {
    pub fn new(field_width: u16, top: u16) -> Self {
        let gap = (field_width - COUNT * WIDTH) / (COUNT + 1);
        let blocks = (0..COUNT)
            .flat_map(|index| {
                let left = gap + index * (WIDTH + gap);
                SHAPE.iter().zip(top..).flat_map(move |(line, y)| {
                    line.chars()
                        .zip(left..)
                        .filter(|(block, _)| *block == '#')
                        .map(move |(_, x)| (x, y))
                })
            })
            .collect();
        Self { blocks }
    }

    pub fn hit(&mut self, x: u16, y: u16) -> bool {
        self.blocks.remove(&(x, y))
    }

    pub fn erode(&mut self, x: u16, y: u16, width: u16) {
        for column in x..x + width {
            self.blocks.remove(&(column, y));
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item = (u16, u16)> + '_ {
        self.blocks.iter().copied()
    }
}
