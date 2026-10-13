use ratatui::layout::{Constraint, Rect, Size};

pub fn centered(area: Rect, size: Size) -> Rect {
    area.centered(
        Constraint::Length(size.width),
        Constraint::Length(size.height),
    )
}

pub fn fits(area: Rect, size: Size) -> bool {
    area.width >= size.width && area.height >= size.height
}
