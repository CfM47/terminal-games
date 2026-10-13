use crate::audio::{Duty, Note, Tune, Voice, pitch::*};

pub const PADDLE: Tune = Tune {
    voices: &[Voice::pulse(Duty::Half, 0.08, &[Note::tone(A4, 35)])],
};

pub const WALL: Tune = Tune {
    voices: &[Voice::triangle(0.15, &[Note::tone(E4, 25)])],
};

pub const POINT: Tune = Tune {
    voices: &[
        Voice::pulse(
            Duty::Eighth,
            0.08,
            &[Note::tone(E5, 60), Note::tone(A4, 140)],
        ),
        Voice::noise(0.04, &[Note::tone(3_000.0, 200)]),
    ],
};

pub const MATCH_WON: Tune = Tune {
    voices: &[
        Voice::pulse(
            Duty::Quarter,
            0.12,
            &[
                Note::tone(G4, 80),
                Note::tone(C5, 80),
                Note::tone(E5, 80),
                Note::tone(G5, 80),
                Note::rest(40),
                Note::tone(E5, 80),
                Note::tone(G5, 300),
            ],
        ),
        Voice::triangle(
            0.12,
            &[Note::tone(C3, 320), Note::rest(40), Note::tone(C4, 380)],
        ),
    ],
};
