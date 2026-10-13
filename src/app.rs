use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Position, Rect},
};

use crate::{
    audio::Speaker,
    game::GameEntry,
    input::{MouseCapture, ReleaseReporting},
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
    viewport: Rect,
    running: bool,
}

impl App {
    pub fn new(catalog: &'static [GameEntry]) -> Self {
        Self {
            menu: Menu::new(catalog),
            screen: Screen::Menu,
            scores: HighScores::load(),
            speaker: Speaker::open(),
            viewport: Rect::default(),
            running: true,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let _release_reporting = ReleaseReporting::enable();
        let _mouse_capture = MouseCapture::enable();
        let mut last_tick = Instant::now();
        while self.running {
            self.viewport = terminal.draw(|frame| self.render(frame))?.area;
            self.handle_events(last_tick + FRAME)?;
            self.update(last_tick.elapsed());
            last_tick = Instant::now();
        }
        Ok(())
    }

    fn handle_events(&mut self, deadline: Instant) -> io::Result<()> {
        while event::poll(deadline.saturating_duration_since(Instant::now()))? {
            match event::read()? {
                Event::Key(key) => self.handle_key(key),
                Event::Mouse(mouse) => self.handle_mouse(mouse),
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }
        if key.kind == KeyEventKind::Release {
            if let Screen::Playing(session) = &mut self.screen {
                session.handle_key_release(key.code);
            }
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

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if let (Screen::Playing(session), MouseEventKind::Down(button)) =
            (&mut self.screen, mouse.kind)
        {
            let position = Position::new(mouse.column, mouse.row);
            session.handle_click(position, button, self.viewport, &self.speaker);
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
