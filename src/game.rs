use std::time::Duration;

use crossterm::event::KeyCode;
use ratatui::{Frame, layout::Rect, layout::Size};

use crate::audio::{Speaker, Tune};

pub trait Game {
    /// Area the game needs to render its playfield.
    fn size(&self) -> Size;
    fn handle_key(&mut self, key: KeyCode, speaker: &Speaker);
    fn update(&mut self, elapsed: Duration, speaker: &Speaker);
    fn render(&self, frame: &mut Frame, area: Rect);
    fn score(&self) -> u32;
    fn is_over(&self) -> bool;
}

pub struct GameEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub controls: &'static str,
    pub music: Option<&'static Tune>,
    pub launch: fn() -> Box<dyn Game>,
}
