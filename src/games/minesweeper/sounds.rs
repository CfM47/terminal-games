use crate::audio::{Duty, Note, Tune, Voice, pitch::*};

pub const OPEN: Tune = Tune {
    voices: &[Voice::triangle(0.15, &[Note::tone(G4, 25)])],
};

pub const OPEN_AREA: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.07,
        &[Note::tone(C5, 25), Note::tone(E5, 25), Note::tone(G5, 40)],
    )],
};

pub const FLAG: Tune = Tune {
    voices: &[Voice::pulse(Duty::Eighth, 0.08, &[Note::tone(E5, 35)])],
};

pub const UNFLAG: Tune = Tune {
    voices: &[Voice::pulse(Duty::Eighth, 0.08, &[Note::tone(C5, 35)])],
};

pub const EXPLOSION: Tune = Tune {
    voices: &[
        Voice::noise(0.14, &[Note::tone(900.0, 700)]),
        Voice::triangle(0.15, &[Note::tone(C2, 150), Note::tone(A2, 300)]),
    ],
};
