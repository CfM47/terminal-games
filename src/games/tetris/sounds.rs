use crate::audio::{Duty, Note, Tune, Voice, pitch::*};

const EIGHTH: u32 = 250;
const KICK: f32 = 1_000.0;
const SNARE: f32 = 6_000.0;
const HAT: f32 = 30_000.0;

const fn eighths(frequency: f32, count: u32) -> Note {
    Note::tone(frequency, count * EIGHTH)
}

const fn silence(count: u32) -> Note {
    Note::rest(count * EIGHTH)
}

#[rustfmt::skip]
const MELODY: &[Note] = &[
    eighths(E5, 2), eighths(B4, 1), eighths(C5, 1), eighths(D5, 2), eighths(C5, 1), eighths(B4, 1),
    eighths(A4, 2), eighths(A4, 1), eighths(C5, 1), eighths(E5, 2), eighths(D5, 1), eighths(C5, 1),
    eighths(B4, 2), eighths(B4, 1), eighths(C5, 1), eighths(D5, 2), eighths(E5, 2),
    eighths(C5, 2), eighths(A4, 2), eighths(A4, 2), eighths(B3, 1), eighths(C4, 1),
    silence(2), eighths(D5, 2), eighths(F5, 1), eighths(A5, 2), silence(1),
    eighths(G5, 1), eighths(F5, 1), eighths(E5, 4), eighths(C5, 1), eighths(E5, 3),
    eighths(D5, 1), eighths(C5, 1), eighths(B4, 3), eighths(C5, 1),
    eighths(D5, 2), eighths(E5, 2), eighths(C5, 2), eighths(A4, 2),
    eighths(A4, 2), silence(2),
    eighths(E5, 4), eighths(C5, 4),
    eighths(D5, 4), eighths(B4, 4),
    eighths(C5, 4), eighths(A4, 4),
    eighths(GS4, 4), eighths(B4, 4),
    eighths(E5, 4), eighths(C5, 4),
    eighths(D5, 4), eighths(B4, 4),
    eighths(C5, 2), eighths(E5, 2), eighths(A5, 4),
    eighths(GS5, 8),
];

#[rustfmt::skip]
const BASS: &[Note] = &[
    eighths(E3, 1), eighths(E4, 1), eighths(E3, 1), eighths(E4, 1), eighths(E3, 1), eighths(E4, 1), eighths(E3, 1), eighths(E4, 1),
    eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1),
    eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1),
    eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(B2, 1), eighths(C3, 1),
    eighths(D3, 1), eighths(D4, 1), eighths(D3, 1), eighths(D4, 1), eighths(D3, 1), eighths(D4, 1), eighths(D3, 1), eighths(D4, 1),
    eighths(D3, 1), eighths(D4, 1), eighths(C3, 1), eighths(C4, 1), eighths(C3, 1), eighths(C4, 1), eighths(C3, 1), eighths(C4, 1),
    eighths(C3, 1), eighths(C4, 1), eighths(C3, 1), eighths(C4, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1),
    eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1),
    eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1),
    eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1),
    eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1),
    eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1),
    eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1),
    eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1),
    eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1),
    eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1), eighths(A2, 1), eighths(A3, 1),
    eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1), eighths(GS2, 1), eighths(GS3, 1),
];

#[rustfmt::skip]
const DRUMS: &[Note] = &[
    eighths(KICK, 1), eighths(HAT, 1), eighths(SNARE, 1), eighths(HAT, 1),
];

pub const KOROBEINIKI: Tune = Tune {
    voices: &[
        Voice::pulse(Duty::Quarter, 0.05, MELODY),
        Voice::triangle(0.09, BASS),
        Voice::noise(0.035, DRUMS),
    ],
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
