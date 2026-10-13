use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Paragraph, Wrap},
};

use crate::{
    audio::{Speaker, tunes},
    game::GameEntry,
    layout,
    scores::{HighScores, Ranking},
};

const TITLE: &str = "TERMINAL GAMES";
const RAINBOW: [Color; 6] = [
    Color::LightRed,
    Color::LightYellow,
    Color::LightGreen,
    Color::LightCyan,
    Color::LightBlue,
    Color::LightMagenta,
];
const WIDTH: u16 = 56;
const DETAILS_HEIGHT: u16 = 12;
const FIXED_HEIGHT: u16 = 4 + DETAILS_HEIGHT;

pub enum MenuAction {
    Stay,
    Launch(&'static GameEntry),
    Quit,
}

#[derive(Clone, Copy)]
enum Item {
    Game(&'static GameEntry),
    Quit,
}

impl Item {
    fn label(self) -> &'static str {
        match self {
            Item::Game(entry) => entry.title,
            Item::Quit => "Quit",
        }
    }

    fn matches(self, filter: &str) -> bool {
        self.label().to_lowercase().contains(&filter.to_lowercase())
    }
}

/// Game picker modeled after the GitHub CLI select prompt.
pub struct Menu {
    catalog: &'static [GameEntry],
    filter: String,
    cursor: usize,
}

impl Menu {
    pub fn new(catalog: &'static [GameEntry]) -> Self {
        Self {
            catalog,
            filter: String::new(),
            cursor: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, speaker: &Speaker) -> MenuAction {
        match key {
            KeyCode::Up => self.move_cursor(-1, speaker),
            KeyCode::Down | KeyCode::Tab => self.move_cursor(1, speaker),
            KeyCode::Enter => return self.confirm(speaker),
            KeyCode::Esc if self.filter.is_empty() => return MenuAction::Quit,
            KeyCode::Esc => self.set_filter(String::new()),
            KeyCode::Backspace => {
                let mut filter = self.filter.clone();
                filter.pop();
                self.set_filter(filter);
            }
            KeyCode::Char(character) => self.set_filter(format!("{}{character}", self.filter)),
            _ => {}
        }
        MenuAction::Stay
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, scores: &HighScores) {
        let items = self.items();
        let room = area.height.saturating_sub(FIXED_HEIGHT).max(1);
        let list_height = (items.len().max(1) as u16).min(room);
        let size = Size::new(WIDTH, FIXED_HEIGHT + list_height);
        let [banner, _, prompt, list, _, details] =
            layout::centered(area, size).layout(&Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(list_height),
                Constraint::Length(1),
                Constraint::Length(DETAILS_HEIGHT),
            ]));

        frame.render_widget(rainbow_banner().centered(), banner);
        frame.render_widget(self.prompt_line(), prompt);
        frame.render_widget(self.list(&items, list_height), list);
        frame.render_widget(details_panel(self.selected(), scores), details);
    }

    fn items(&self) -> Vec<Item> {
        self.catalog
            .iter()
            .map(Item::Game)
            .chain([Item::Quit])
            .filter(|item| item.matches(&self.filter))
            .collect()
    }

    fn selected(&self) -> Option<Item> {
        self.items().get(self.cursor).copied()
    }

    fn move_cursor(&mut self, delta: isize, speaker: &Speaker) {
        let count = self.items().len();
        if count == 0 {
            return;
        }
        self.cursor = (self.cursor as isize + delta).rem_euclid(count as isize) as usize;
        speaker.play(&tunes::MENU_MOVE);
    }

    fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.cursor = 0;
    }

    fn confirm(&mut self, speaker: &Speaker) -> MenuAction {
        match self.selected() {
            Some(Item::Game(entry)) => {
                speaker.play(&tunes::MENU_SELECT);
                MenuAction::Launch(entry)
            }
            Some(Item::Quit) => MenuAction::Quit,
            None => MenuAction::Stay,
        }
    }

    fn prompt_line(&self) -> Line<'_> {
        let mut spans = vec!["? ".green().bold(), "Choose a game: ".bold()];
        if self.filter.is_empty() {
            spans.push("[Use arrows to move, type to filter]".cyan());
        } else {
            spans.push(self.filter.as_str().into());
        }
        Line::from(spans)
    }

    /// Visible slice of the items, scrolled just enough to keep the cursor in view.
    fn list(&self, items: &[Item], height: u16) -> Paragraph<'static> {
        if items.is_empty() {
            return Paragraph::new("  No results".dark_gray().italic());
        }
        let first_visible = self.cursor.saturating_sub(usize::from(height) - 1);
        let lines: Vec<Line> = items
            .iter()
            .enumerate()
            .skip(first_visible)
            .take(usize::from(height))
            .map(|(index, item)| {
                if index == self.cursor {
                    Line::from(format!("> {}", item.label())).cyan().bold()
                } else {
                    Line::from(format!("  {}", item.label()))
                }
            })
            .collect();
        Paragraph::new(lines)
    }
}

fn rainbow_banner() -> Line<'static> {
    let mut colors = RAINBOW.iter().cycle();
    TITLE
        .chars()
        .map(|letter| match letter {
            ' ' => Span::raw("  "),
            _ => Span::styled(
                format!("{letter} "),
                Style::new().fg(*colors.next().unwrap()).bold(),
            ),
        })
        .collect()
}

fn details_panel(item: Option<Item>, scores: &HighScores) -> Paragraph<'static> {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dark_gray())
        .padding(Padding::horizontal(1));
    let Some(Item::Game(entry)) = item else {
        return Paragraph::new("Leave the arcade.".dark_gray()).block(block);
    };

    let mut lines = vec![
        Line::from(entry.description),
        Line::from(vec!["Controls  ".dark_gray(), entry.controls.into()]),
        Line::default(),
    ];
    if entry.ranking == Ranking::Unranked {
        lines.push(Line::from("Unranked mode".dark_gray().italic()));
    } else {
        lines.push(Line::from(entry.ranking.board_title().yellow().bold()));
        lines.extend(score_lines(scores.top(entry.id), entry.ranking));
    }

    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(block.title(Line::from(format!(" {} ", entry.title)).bold()))
}

fn score_lines(board: &[u32], ranking: Ranking) -> Vec<Line<'static>> {
    if board.is_empty() {
        return vec![Line::from("No scores yet".dark_gray().italic())];
    }
    board
        .iter()
        .enumerate()
        .map(|(rank, &score)| Line::from(format!("{}. {:>7}", rank + 1, ranking.format(score))))
        .collect()
}
