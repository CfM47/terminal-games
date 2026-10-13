pub mod pitch;
mod synth;
pub mod tunes;

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};

pub use synth::{Duty, Note, Tune, Voice};
use synth::{Synth, Tempo};

/// Looping background tune that stops when dropped.
pub struct Music {
    player: Player,
    tempo: Tempo,
}

impl Music {
    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    pub fn set_tempo(&self, ratio: f32) {
        self.tempo.set(ratio);
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
            sink.mixer().add(Synth::once(tune));
        }
    }

    pub fn play_looped(&self, tune: &Tune) -> Option<Music> {
        let sink = self.sink.as_ref()?;
        let tempo = Tempo::default();
        let player = Player::connect_new(sink.mixer());
        player.append(Synth::looped(tune, tempo.clone()));
        Some(Music { player, tempo })
    }
}
