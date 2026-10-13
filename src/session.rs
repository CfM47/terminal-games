use std::time::Duration;

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::{
    audio::{Music, Speaker, tunes},
    game::{Game, GameEntry},
    layout,
    scores::HighScores,
};

pub enum SessionAction {
    Stay,
    Exit,
}

struct Outcome {
    score: u32,
    rank: Option<usize>,
}

/// One run of a game, wrapped with the HUD, pausing and score submission.
pub struct Session {
    entry: &'static GameEntry,
    game: Box<dyn Game>,
    music: Option<Music>,
    paused: bool,
    outcome: Option<Outcome>,
}

impl Session {
    pub fn start(entry: &'static GameEntry, speaker: &Speaker) -> Self {
        Self {
            entry,
            game: (entry.launch)(),
            music: entry.music.and_then(|tune| speaker.play_looped(tune)),
            paused: false,
            outcome: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, speaker: &Speaker) -> SessionAction {
        match key {
            KeyCode::Esc => return SessionAction::Exit,
            KeyCode::Enter | KeyCode::Char('r') if self.is_finished() => {
                *self = Self::start(self.entry, speaker);
            }
            _ if self.is_finished() => {}
            KeyCode::Char('p') => self.toggle_pause(),
            _ if self.paused => {}
            _ => self.game.handle_key(key, speaker),
        }
        SessionAction::Stay
    }

    pub fn update(&mut self, elapsed: Duration, speaker: &Speaker, scores: &mut HighScores) {
        if self.paused || self.is_finished() {
            return;
        }
        self.game.update(elapsed, speaker);
        if let Some(music) = &self.music {
            music.set_tempo(self.game.music_tempo());
        }
        if self.game.is_over() {
            self.finish(speaker, scores);
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, scores: &HighScores) {
        let playfield = self.game.size();
        let footer = self.footer();
        let width = playfield.width.max(footer.width() as u16);
        let size = Size::new(width, playfield.height + 2);
        if !layout::fits(area, size) {
            render_too_small(frame, area, size);
            return;
        }

        let [hud_row, board_row, footer_row] =
            layout::centered(area, size).layout(&Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(playfield.height),
                Constraint::Length(1),
            ]));
        let board = layout::centered(board_row, playfield);
        self.render_hud(
            frame,
            layout::centered(hud_row, Size::new(playfield.width, 1)),
            scores,
        );
        self.game.render(frame, board);
        frame.render_widget(footer, footer_row);

        if let Some(outcome) = &self.outcome {
            render_popup(frame, board, outcome.lines());
        } else if self.paused {
            render_popup(frame, board, paused_lines());
        }
    }

    fn is_finished(&self) -> bool {
        self.outcome.is_some()
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if let Some(music) = &self.music {
            if self.paused {
                music.pause()
            } else {
                music.resume()
            }
        }
    }

    fn finish(&mut self, speaker: &Speaker, scores: &mut HighScores) {
        self.music = None;
        let score = self.game.score();
        let rank = scores.submit(self.entry.id, score);
        speaker.play(if rank == Some(0) {
            &tunes::NEW_RECORD
        } else {
            &tunes::GAME_OVER
        });
        self.outcome = Some(Outcome { score, rank });
    }

    fn render_hud(&self, frame: &mut Frame, area: Rect, scores: &HighScores) {
        let best = scores
            .best(self.entry.id)
            .unwrap_or(0)
            .max(self.game.score());
        let stats = Line::from(vec![
            "Score ".dark_gray(),
            self.game.score().to_string().bold(),
            "  Best ".dark_gray(),
            best.to_string().yellow().bold(),
        ]);
        frame.render_widget(
            Line::from(self.entry.title.to_uppercase()).green().bold(),
            area,
        );
        frame.render_widget(stats.right_aligned(), area);
    }

    fn footer(&self) -> Line<'static> {
        Line::from(format!("{} · p pause · esc menu", self.entry.controls))
            .dark_gray()
            .centered()
    }
}

impl Outcome {
    fn lines(&self) -> Vec<Line<'static>> {
        let verdict = match self.rank {
            Some(0) => "New record!".yellow().bold(),
            Some(rank) => format!("#{} on the board", rank + 1).yellow(),
            None => "".into(),
        };
        vec![
            Line::from("GAME OVER".red().bold()),
            Line::default(),
            Line::from(format!("Score {}", self.score)),
            Line::from(verdict),
            Line::default(),
            Line::from("enter play again · esc menu".dark_gray()),
        ]
    }
}

fn paused_lines() -> Vec<Line<'static>> {
    vec![
        Line::from("PAUSED".cyan().bold()),
        Line::default(),
        Line::from("p resume · esc menu".dark_gray()),
    ]
}

fn render_popup(frame: &mut Frame, area: Rect, lines: Vec<Line<'static>>) {
    let width = lines.iter().map(Line::width).max().unwrap_or(0) as u16 + 6;
    let popup = layout::centered(area, Size::new(width, lines.len() as u16 + 2)).clamp(area);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().yellow());
    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).centered().block(block), popup);
}

fn render_too_small(frame: &mut Frame, area: Rect, needed: Size) {
    let message = Paragraph::new(vec![
        Line::from("Terminal too small".red().bold()),
        Line::from(format!("needs {}×{}", needed.width, needed.height).dark_gray()),
    ])
    .centered();
    frame.render_widget(message, layout::centered(area, Size::new(area.width, 2)));
}
