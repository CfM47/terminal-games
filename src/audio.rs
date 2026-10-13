use std::num::NonZero;

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, Source, buffer::SamplesBuffer};

const SAMPLE_RATE: NonZero<u32> = NonZero::new(44_100).unwrap();
const MONO: NonZero<u16> = NonZero::new(1).unwrap();

pub mod pitch {
    pub const C4: f32 = 261.63;
    pub const E4: f32 = 329.63;
    pub const G4: f32 = 392.00;
    pub const A4: f32 = 440.00;
    pub const B4: f32 = 493.88;
    pub const C5: f32 = 523.25;
    pub const D5: f32 = 587.33;
    pub const E5: f32 = 659.25;
    pub const F5: f32 = 698.46;
    pub const G5: f32 = 783.99;
    pub const A5: f32 = 880.00;
    pub const B5: f32 = 987.77;
    pub const C6: f32 = 1046.50;
    pub const E6: f32 = 1318.51;
}

/// Pulse width of the square wave, the knob that gave each NES channel its timbre.
#[derive(Clone, Copy)]
pub enum Duty {
    Eighth,
    Quarter,
    Half,
}

impl Duty {
    fn ratio(self) -> f32 {
        match self {
            Duty::Eighth => 0.125,
            Duty::Quarter => 0.25,
            Duty::Half => 0.5,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Note {
    frequency: Option<f32>,
    millis: u32,
}

impl Note {
    pub const fn tone(frequency: f32, millis: u32) -> Self {
        Self {
            frequency: Some(frequency),
            millis,
        }
    }

    pub const fn rest(millis: u32) -> Self {
        Self {
            frequency: None,
            millis,
        }
    }

    fn samples(self, duty: Duty, volume: f32) -> impl Iterator<Item = f32> {
        let count = SAMPLE_RATE.get() * self.millis / 1000;
        let audible = count * 7 / 8;
        (0..count).map(move |index| match self.frequency {
            Some(frequency) if index < audible => volume * square_wave(index, frequency, duty),
            _ => 0.0,
        })
    }
}

fn square_wave(index: u32, frequency: f32, duty: Duty) -> f32 {
    let phase = (index as f32 * frequency / SAMPLE_RATE.get() as f32).fract();
    if phase < duty.ratio() { 1.0 } else { -1.0 }
}

pub struct Tune {
    pub duty: Duty,
    pub volume: f32,
    pub notes: &'static [Note],
}

impl Tune {
    fn synthesize(&self) -> SamplesBuffer {
        let samples: Vec<f32> = self
            .notes
            .iter()
            .flat_map(|note| note.samples(self.duty, self.volume))
            .collect();
        SamplesBuffer::new(MONO, SAMPLE_RATE, samples)
    }
}

pub mod tunes {
    use super::{Duty, Note, Tune, pitch::*};

    pub const MENU_MOVE: Tune = Tune {
        duty: Duty::Eighth,
        volume: 0.08,
        notes: &[Note::tone(A5, 25)],
    };

    pub const MENU_SELECT: Tune = Tune {
        duty: Duty::Quarter,
        volume: 0.12,
        notes: &[
            Note::tone(C5, 45),
            Note::tone(E5, 45),
            Note::tone(G5, 45),
            Note::tone(C6, 90),
        ],
    };

    pub const GAME_OVER: Tune = Tune {
        duty: Duty::Half,
        volume: 0.12,
        notes: &[
            Note::tone(G4, 140),
            Note::tone(E4, 140),
            Note::tone(C4, 140),
            Note::rest(40),
            Note::tone(C4, 320),
        ],
    };

    pub const NEW_RECORD: Tune = Tune {
        duty: Duty::Quarter,
        volume: 0.12,
        notes: &[
            Note::tone(C5, 90),
            Note::tone(E5, 90),
            Note::tone(G5, 90),
            Note::rest(60),
            Note::tone(B5, 90),
            Note::tone(E6, 300),
        ],
    };
}

/// Looping background tune that stops when dropped.
pub struct Music {
    player: Player,
}

impl Music {
    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }
}

/// Plays tunes on the default output device, staying silent when none is available.
pub struct Speaker {
    sink: Option<MixerDeviceSink>,
}

impl Speaker {
    pub fn open() -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().ok().map(|mut sink| {
            sink.log_on_drop(false);
            sink
        });
        Self { sink }
    }

    pub fn play(&self, tune: &Tune) {
        if let Some(sink) = &self.sink {
            sink.mixer().add(tune.synthesize());
        }
    }

    pub fn play_looped(&self, tune: &Tune) -> Option<Music> {
        let sink = self.sink.as_ref()?;
        let player = Player::connect_new(sink.mixer());
        player.append(tune.synthesize().repeat_infinite());
        Some(Music { player })
    }
}
