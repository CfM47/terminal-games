use std::io;

use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute, terminal,
};

/// Asks the terminal to report key releases for as long as this guard lives.
pub struct ReleaseReporting;

impl ReleaseReporting {
    pub fn enable() -> Option<Self> {
        let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES;
        let supported = terminal::supports_keyboard_enhancement().unwrap_or(false);
        let enabled =
            supported && execute!(io::stdout(), PushKeyboardEnhancementFlags(flags)).is_ok();
        enabled.then_some(Self)
    }
}

impl Drop for ReleaseReporting {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
    }
}
