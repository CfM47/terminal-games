use crate::audio::{Duty, Note, Tune, Voice, pitch::*};

const EIGHTH: u32 = 125;

const fn eighths(frequency: f32, count: u32) -> Note {
    Note::tone(frequency, count * EIGHTH)
}

#[rustfmt::skip]
const MELODY: &[Note] = &[
    eighths(E5, 2), eighths(B4, 1), eighths(C5, 1), eighths(D5, 2), eighths(C5, 1), eighths(B4, 1),
    eighths(A4, 2), eighths(A4, 1), eighths(C5, 1), eighths(E5, 2), eighths(D5, 1), eighths(C5, 1),
    eighths(B4, 3), eighths(C5, 1), eighths(D5, 2), eighths(E5, 2),
    eighths(C5, 2), eighths(A4, 2), eighths(A4, 4),
    eighths(D5, 3), eighths(F5, 1), eighths(A5, 2), eighths(G5, 1), eighths(F5, 1),
    eighths(E5, 3), eighths(C5, 1), eighths(E5, 2), eighths(D5, 1), eighths(C5, 1),
    eighths(B4, 2), eighths(B4, 1), eighths(C5, 1), eighths(D5, 2), eighths(E5, 2),
    eighths(C5, 2), eighths(A4, 2), eighths(A4, 4),
];

pub const KOROBEINIKI: Tune = Tune {
    voices: &[Voice::pulse(Duty::Quarter, 0.05, MELODY)],
};

pub const ROTATE: Tune = Tune {
    voices: &[Voice::pulse(Duty::Eighth, 0.06, &[Note::tone(A5, 20)])],
};

pub const LOCK: Tune = Tune {
    voices: &[Voice::pulse(Duty::Half, 0.08, &[Note::tone(C4, 35)])],
};

pub const LINE_CLEAR: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.1,
        &[Note::tone(C5, 40), Note::tone(E5, 40), Note::tone(G5, 80)],
    )],
};

pub const FOUR_LINES: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.12,
        &[
            Note::tone(C5, 50),
            Note::tone(E5, 50),
            Note::tone(G5, 50),
            Note::tone(C6, 50),
            Note::tone(E6, 200),
        ],
    )],
};
