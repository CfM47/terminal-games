use super::{Duty, Note, Tune, Voice, pitch::*};

pub const MENU_MOVE: Tune = Tune {
    voices: &[Voice::pulse(Duty::Eighth, 0.08, &[Note::tone(A5, 25)])],
};

pub const MENU_SELECT: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.12,
        &[
            Note::tone(C5, 45),
            Note::tone(E5, 45),
            Note::tone(G5, 45),
            Note::tone(C6, 90),
        ],
    )],
};

pub const GAME_OVER: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Half,
        0.12,
        &[
            Note::tone(G4, 140),
            Note::tone(E4, 140),
            Note::tone(C4, 140),
            Note::rest(40),
            Note::tone(C4, 320),
        ],
    )],
};

pub const NEW_RECORD: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.12,
        &[
            Note::tone(C5, 90),
            Note::tone(E5, 90),
            Note::tone(G5, 90),
            Note::rest(60),
            Note::tone(B5, 90),
            Note::tone(E6, 300),
        ],
    )],
};
