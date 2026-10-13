use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};

use crate::{
    audio::Speaker,
    game::GameEntry,
    menu::{Menu, MenuAction},
    scores::HighScores,
    session::{Session, SessionAction},
};

const FRAME: Duration = Duration::from_millis(16);

enum Screen {
    Menu,
    Playing(Session),
}

pub struct App {
    menu: Menu,
    screen: Screen,
    scores: HighScores,
    speaker: Speaker,
    running: bool,
}

impl App {
    pub fn new(catalog: &'static [GameEntry]) -> Self {
        Self {
            menu: Menu::new(catalog),
            screen: Screen::Menu,
            scores: HighScores::load(),
            speaker: Speaker::open(),
            running: true,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_tick = Instant::now();
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events(last_tick + FRAME)?;
            self.update(last_tick.elapsed());
            last_tick = Instant::now();
        }
        Ok(())
    }

    fn handle_events(&mut self, deadline: Instant) -> io::Result<()> {
        while event::poll(deadline.saturating_duration_since(Instant::now()))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }
        match &mut self.screen {
            Screen::Menu => match self.menu.handle_key(key.code, &self.speaker) {
                MenuAction::Stay => {}
                MenuAction::Launch(entry) => {
                    self.screen = Screen::Playing(Session::start(entry, &self.speaker))
                }
                MenuAction::Quit => self.running = false,
            },
            Screen::Playing(session) => match session.handle_key(key.code, &self.speaker) {
                SessionAction::Stay => {}
                SessionAction::Exit => self.screen = Screen::Menu,
            },
        }
    }

    fn update(&mut self, elapsed: Duration) {
        if let Screen::Playing(session) = &mut self.screen {
            session.update(elapsed, &self.speaker, &mut self.scores);
        }
    }

    fn render(&self, frame: &mut Frame) {
        match &self.screen {
            Screen::Menu => self.menu.render(frame, frame.area(), &self.scores),
            Screen::Playing(session) => session.render(frame, frame.area(), &self.scores),
        }
    }
}
