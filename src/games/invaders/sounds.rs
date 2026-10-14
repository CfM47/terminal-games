use crate::audio::{Duty, Note, Tune, Voice, pitch::*};

pub const MARCH: [Tune; 4] = [
    Tune {
        voices: &[Voice::triangle(0.22, &[Note::tone(G2, 90)])],
    },
    Tune {
        voices: &[Voice::triangle(0.22, &[Note::tone(F2, 90)])],
    },
    Tune {
        voices: &[Voice::triangle(0.22, &[Note::tone(E2, 90)])],
    },
    Tune {
        voices: &[Voice::triangle(0.22, &[Note::tone(D2, 90)])],
    },
];

pub const SHOOT: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Eighth,
        0.07,
        &[Note::tone(C6, 20), Note::tone(G5, 20), Note::tone(E5, 30)],
    )],
};

pub const INVADER_DIES: Tune = Tune {
    voices: &[Voice::noise(0.08, &[Note::tone(4_000.0, 120)])],
};

pub const CANNON_DIES: Tune = Tune {
    voices: &[
        Voice::noise(0.12, &[Note::tone(700.0, 900)]),
        Voice::triangle(0.15, &[Note::tone(C3, 200), Note::tone(C2, 500)]),
    ],
};

pub const UFO: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.04,
        &[
            Note::tone(A5, 60),
            Note::tone(C6, 60),
            Note::tone(A5, 60),
            Note::tone(C6, 60),
        ],
    )],
};

pub const UFO_HIT: Tune = Tune {
    voices: &[Voice::pulse(
        Duty::Quarter,
        0.1,
        &[
            Note::tone(C6, 50),
            Note::tone(G5, 50),
            Note::tone(E6, 50),
            Note::tone(C6, 150),
        ],
    )],
};

pub const WAVE_CLEARED: Tune = Tune {
    voices: &[
        Voice::pulse(
            Duty::Quarter,
            0.12,
            &[
                Note::tone(E5, 80),
                Note::tone(G5, 80),
                Note::tone(C6, 80),
                Note::rest(40),
                Note::tone(G5, 80),
                Note::tone(E6, 300),
            ],
        ),
        Voice::triangle(
            0.12,
            &[Note::tone(C3, 280), Note::rest(40), Note::tone(C4, 380)],
        ),
    ],
};
