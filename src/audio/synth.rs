use std::{
    num::NonZero,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use rodio::{ChannelCount, SampleRate, Source};

const SAMPLE_RATE: SampleRate = NonZero::new(44_100).unwrap();
const MONO: ChannelCount = NonZero::new(1).unwrap();
const TRIANGLE_STEPS: f32 = 16.0;
const NOISE_SEED: u16 = 1;

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
enum Waveform {
    Pulse(Duty),
    Triangle,
    Noise,
}

/// A pitch held for a length of time; for noise voices the pitch is the noise clock rate.
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

    fn seconds(self) -> f32 {
        self.millis as f32 / 1000.0
    }
}

pub struct Voice {
    waveform: Waveform,
    volume: f32,
    notes: &'static [Note],
}

impl Voice {
    pub const fn pulse(duty: Duty, volume: f32, notes: &'static [Note]) -> Self {
        Self {
            waveform: Waveform::Pulse(duty),
            volume,
            notes,
        }
    }

    pub const fn triangle(volume: f32, notes: &'static [Note]) -> Self {
        Self {
            waveform: Waveform::Triangle,
            volume,
            notes,
        }
    }

    pub const fn noise(volume: f32, notes: &'static [Note]) -> Self {
        Self {
            waveform: Waveform::Noise,
            volume,
            notes,
        }
    }
}

/// Voices played together; each one loops on its own length when the tune repeats.
pub struct Tune {
    pub voices: &'static [Voice],
}

/// Playback speed shared between the game and the audio thread.
#[derive(Clone)]
pub struct Tempo(Arc<AtomicU32>);

impl Tempo {
    pub fn set(&self, ratio: f32) {
        self.0.store(ratio.to_bits(), Ordering::Relaxed);
    }

    fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
}

impl Default for Tempo {
    fn default() -> Self {
        Self(Arc::new(AtomicU32::new(1.0_f32.to_bits())))
    }
}

struct Channel {
    voice: &'static Voice,
    note_index: usize,
    note_elapsed: f32,
    phase: f32,
    noise: u16,
}

impl Channel {
    fn new(voice: &'static Voice) -> Self {
        Self {
            voice,
            note_index: 0,
            note_elapsed: 0.0,
            phase: 0.0,
            noise: NOISE_SEED,
        }
    }

    fn next_sample(&mut self, score_step: f32, looping: bool) -> Option<f32> {
        let note = *self.voice.notes.get(self.note_index)?;
        let sample = note.frequency.map_or(0.0, |frequency| {
            self.oscillate(frequency) * self.envelope(note)
        });
        self.note_elapsed += score_step;
        if self.note_elapsed >= note.seconds() {
            self.note_elapsed -= note.seconds();
            self.note_index += 1;
            if looping && self.note_index == self.voice.notes.len() {
                self.note_index = 0;
            }
        }
        Some(sample * self.voice.volume)
    }

    fn oscillate(&mut self, frequency: f32) -> f32 {
        self.phase += frequency / SAMPLE_RATE.get() as f32;
        let cycles = self.phase.floor();
        self.phase -= cycles;
        match self.voice.waveform {
            Waveform::Pulse(duty) => {
                if self.phase < duty.ratio() {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Triangle => {
                let ramp = 1.0 - 4.0 * (self.phase - 0.5).abs();
                (ramp * TRIANGLE_STEPS / 2.0).round() / (TRIANGLE_STEPS / 2.0)
            }
            Waveform::Noise => {
                (0..cycles as u32).for_each(|_| self.shift_noise());
                if self.noise & 1 == 0 { 1.0 } else { -1.0 }
            }
        }
    }

    /// Fifteen-bit linear feedback shift register, the same one the NES noise channel used.
    fn shift_noise(&mut self) {
        let feedback = (self.noise ^ (self.noise >> 1)) & 1;
        self.noise = (self.noise >> 1) | (feedback << 14);
    }

    fn envelope(&self, note: Note) -> f32 {
        let progress = self.note_elapsed / note.seconds();
        match self.voice.waveform {
            Waveform::Noise => (1.0 - progress).max(0.0).powi(2),
            _ if progress < 7.0 / 8.0 => 1.0,
            _ => 0.0,
        }
    }
}

/// Real-time mixer of a tune's voices whose speed follows a shared tempo without shifting pitch.
pub struct Synth {
    channels: Vec<Channel>,
    tempo: Tempo,
    looping: bool,
}

impl Synth {
    pub fn once(tune: &Tune) -> Self {
        Self::new(tune, Tempo::default(), false)
    }

    pub fn looped(tune: &Tune, tempo: Tempo) -> Self {
        Self::new(tune, tempo, true)
    }

    fn new(tune: &Tune, tempo: Tempo, looping: bool) -> Self {
        Self {
            channels: tune.voices.iter().map(Channel::new).collect(),
            tempo,
            looping,
        }
    }
}

impl Iterator for Synth {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let score_step = self.tempo.get() / SAMPLE_RATE.get() as f32;
        self.channels
            .iter_mut()
            .filter_map(|channel| channel.next_sample(score_step, self.looping))
            .reduce(|mix, sample| mix + sample)
    }
}

impl Source for Synth {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        MONO
    }

    fn sample_rate(&self) -> SampleRate {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
