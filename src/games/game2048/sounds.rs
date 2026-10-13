use crate::audio::{Duty, Note, Tune, Voice, pitch::*};

pub const SLIDE: Tune = Tune {
    voices: &[Voice::triangle(0.12, &[Note::tone(E4, 30)])],
};

pub const MERGE: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.08,
        &[Note::tone(C5, 30), Note::tone(G5, 50)],
    )],
};

pub const GOAL: Tune = Tune {
    voices: &[
        Voice::pulse(
            Duty::Quarter,
            0.12,
            &[
                Note::tone(C5, 90),
                Note::tone(E5, 90),
                Note::tone(G5, 90),
                Note::tone(C6, 90),
                Note::rest(50),
                Note::tone(B5, 90),
                Note::tone(C6, 120),
                Note::tone(E6, 400),
            ],
        ),
        Voice::triangle(
            0.12,
            &[
                Note::tone(C3, 360),
                Note::rest(50),
                Note::tone(G4, 210),
                Note::tone(C4, 400),
            ],
        ),
    ],
};
