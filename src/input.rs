use std::{collections::HashMap, io, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    event::{
        KeyCode, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute, terminal,
};

const FALLBACK_HOLD: Duration = Duration::from_millis(120);

/// Tracks held keys, falling back to a short timeout per press on terminals that never report releases.
#[derive(Default)]
pub struct HeldKeys {
    expiries: HashMap<KeyCode, Duration>,
    reports_releases: bool,
}

impl HeldKeys {
    pub fn press(&mut self, key: KeyCode) {
        self.expiries.insert(key, FALLBACK_HOLD);
    }

    pub fn release(&mut self, key: KeyCode) {
        self.reports_releases = true;
        self.expiries.remove(&key);
    }

    pub fn tick(&mut self, elapsed: Duration) {
        if self.reports_releases {
            return;
        }
        self.expiries.retain(|_, remaining| {
            *remaining = remaining.saturating_sub(elapsed);
            !remaining.is_zero()
        });
    }

    pub fn is_held(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|key| self.expiries.contains_key(key))
    }
}

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

/// Captures mouse events for as long as this guard lives.
pub struct MouseCapture;

impl MouseCapture {
    pub fn enable() -> Option<Self> {
        execute!(io::stdout(), EnableMouseCapture)
            .ok()
            .map(|()| Self)
    }
}

impl Drop for MouseCapture {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture);
    }
}
