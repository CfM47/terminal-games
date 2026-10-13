use crate::audio::{Duty, Note, Tune, Voice, pitch::*};

pub const PADDLE: Tune = Tune {
    voices: &[Voice::pulse(Duty::Half, 0.08, &[Note::tone(C5, 30)])],
};

pub const WALL: Tune = Tune {
    voices: &[Voice::triangle(0.15, &[Note::tone(G4, 25)])],
};

pub const BRICK: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.08,
        &[Note::tone(E5, 25), Note::tone(B5, 35)],
    )],
};

pub const LOST_BALL: Tune = Tune {
    voices: &[
        Voice::pulse(
            Duty::Half,
            0.1,
            &[Note::tone(E4, 90), Note::tone(C4, 90), Note::tone(A3, 220)],
        ),
        Voice::noise(0.05, &[Note::tone(2_000.0, 400)]),
    ],
};

pub const WALL_CLEARED: Tune = Tune {
    voices: &[
        Voice::pulse(
            Duty::Quarter,
            0.12,
            &[
                Note::tone(C5, 80),
                Note::tone(E5, 80),
                Note::tone(G5, 80),
                Note::tone(C6, 80),
                Note::rest(40),
                Note::tone(G5, 80),
                Note::tone(C6, 300),
            ],
        ),
        Voice::triangle(
            0.12,
            &[Note::tone(C3, 320), Note::rest(40), Note::tone(C4, 380)],
        ),
    ],
};
