use ratatui::{buffer::Buffer, style::Style};

const GLYPH_WIDTH: u16 = 3;
const PIXEL_WIDTH: u16 = 2;
const SPACING: u16 = 2;

#[rustfmt::skip]
const GLYPHS: [[&str; 5]; 10] = [
    ["###", "# #", "# #", "# #", "###"],
    ["  #", "  #", "  #", "  #", "  #"],
    ["###", "  #", "###", "#  ", "###"],
    ["###", "  #", "###", "  #", "###"],
    ["# #", "# #", "###", "  #", "  #"],
    ["###", "#  ", "###", "  #", "###"],
    ["###", "#  ", "###", "# #", "###"],
    ["###", "  #", "  #", "  #", "  #"],
    ["###", "# #", "###", "# #", "###"],
    ["###", "# #", "###", "  #", "###"],
];

/// Draws a number in a large block font horizontally centered on `center_x`.
pub fn draw(buffer: &mut Buffer, center_x: u16, top: u16, number: u32, style: Style) {
    let digits: Vec<usize> = number
        .to_string()
        .bytes()
        .map(|digit| usize::from(digit - b'0'))
        .collect();
    let advance = GLYPH_WIDTH * PIXEL_WIDTH + SPACING;
    let width = advance * digits.len() as u16 - SPACING;
    let left = center_x.saturating_sub(width / 2);

    for (position, digit) in digits.into_iter().enumerate() {
        let glyph_left = left + advance * position as u16;
        for (row, line) in (0..).zip(GLYPHS[digit]) {
            for (column, pixel) in (0..).zip(line.chars()) {
                if pixel == '#' {
                    buffer.set_string(glyph_left + column * PIXEL_WIDTH, top + row, "██", style);
                }
            }
        }
    }
}
