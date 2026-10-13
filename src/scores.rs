use std::{collections::BTreeMap, fs, path::PathBuf};

const BOARD_LENGTH: usize = 5;

type Boards = BTreeMap<String, Vec<u32>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ranking {
    HighestScore,
    FastestTime,
    Unranked,
}

impl Ranking {
    pub fn label(self) -> &'static str {
        match self {
            Ranking::FastestTime => "Time",
            Ranking::HighestScore | Ranking::Unranked => "Score",
        }
    }

    pub fn board_title(self) -> &'static str {
        match self {
            Ranking::FastestTime => "Best times",
            Ranking::HighestScore | Ranking::Unranked => "High scores",
        }
    }

    pub fn format(self, value: u32) -> String {
        match self {
            Ranking::FastestTime => {
                let tenths = value / 100;
                format!("{}:{:02}.{}", tenths / 600, tenths / 10 % 60, tenths % 10)
            }
            Ranking::HighestScore | Ranking::Unranked => value.to_string(),
        }
    }

    fn keeps_ahead(self, existing: u32, newcomer: u32) -> bool {
        match self {
            Ranking::FastestTime => existing <= newcomer,
            Ranking::HighestScore | Ranking::Unranked => existing >= newcomer,
        }
    }
}

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

    /// Records a result and returns its zero-based rank when it makes the board.
    pub fn submit(&mut self, game: &str, ranking: Ranking, score: u32) -> Option<usize> {
        if score == 0 || ranking == Ranking::Unranked {
            return None;
        }
        let board = self.boards.entry(game.to_owned()).or_default();
        let rank = board.partition_point(|&existing| ranking.keeps_ahead(existing, score));
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
