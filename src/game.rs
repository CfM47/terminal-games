use std::time::Duration;

use crossterm::event::{KeyCode, MouseButton};
use ratatui::{
    Frame,
    layout::{Position, Rect, Size},
};

use crate::{
    audio::{Speaker, Tune},
    scores::Ranking,
};

pub trait Game {
    /// Area the game needs to render its playfield.
    fn size(&self) -> Size;
    fn handle_key(&mut self, key: KeyCode, speaker: &Speaker);

    fn handle_key_release(&mut self, _key: KeyCode) {}

    /// Mouse press at a position relative to the game's area.
    fn handle_click(&mut self, _position: Position, _button: MouseButton, _speaker: &Speaker) {}
    fn update(&mut self, _elapsed: Duration, _speaker: &Speaker) {}
    fn render(&self, frame: &mut Frame, area: Rect);
    fn score(&self) -> u32;
    fn is_over(&self) -> bool;

    /// Result submitted to the high scores when the game ends, if it earned one.
    fn record(&self) -> Option<u32> {
        Some(self.score())
    }

    /// Whether the game ended in victory, which plays a celebration instead of the game over tune.
    fn is_won(&self) -> bool {
        false
    }

    /// Closing line for the game over screen, such as who won.
    fn summary(&self) -> Option<String> {
        None
    }

    /// Playback speed of the background music relative to its written tempo.
    fn music_tempo(&self) -> f32 {
        1.0
    }
}

pub struct GameEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub controls: &'static str,
    pub music: Option<&'static Tune>,
    pub ranking: Ranking,
    pub launch: fn() -> Box<dyn Game>,
}
