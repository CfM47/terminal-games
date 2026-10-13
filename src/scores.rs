use std::{collections::BTreeMap, fs, path::PathBuf};

const BOARD_LENGTH: usize = 5;

type Boards = BTreeMap<String, Vec<u32>>;

/// Top scores per game, persisted as TOML in the user's data directory.
pub struct HighScores {
    path: Option<PathBuf>,
    boards: Boards,
}

impl HighScores {
    pub fn load() -> Self {
        let path = dirs::data_dir().map(|dir| dir.join("terminal-games").join("scores.toml"));
        let boards = path
            .as_ref()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| toml::from_str(&contents).ok())
            .unwrap_or_default();
        Self { path, boards }
    }

    pub fn top(&self, game: &str) -> &[u32] {
        self.boards.get(game).map_or(&[], Vec::as_slice)
    }

    pub fn best(&self, game: &str) -> Option<u32> {
        self.top(game).first().copied()
    }

    /// Records a score and returns its zero-based rank when it makes the board.
    pub fn submit(&mut self, game: &str, score: u32) -> Option<usize> {
        if score == 0 {
            return None;
        }
        let board = self.boards.entry(game.to_owned()).or_default();
        let rank = board.partition_point(|&existing| existing >= score);
        if rank >= BOARD_LENGTH {
            return None;
        }
        board.insert(rank, score);
        board.truncate(BOARD_LENGTH);
        self.save();
        Some(rank)
    }

    fn save(&self) {
        let Some(path) = &self.path else { return };
        let Ok(contents) = toml::to_string(&self.boards) else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir).and_then(|()| fs::write(path, contents));
        }
    }
}
