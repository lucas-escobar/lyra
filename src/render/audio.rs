use crate::compose::{DirectionType, MeasureItem, Part};
use std::f64::consts::PI;

pub struct RenderContext {
    pub sample_rate: usize,
}

pub trait Instrument {
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<f64>;
}

pub trait Envelope: Send + Sync {
    fn amplitude(&self, t: f64, note_duration: f64) -> f64;
}

pub struct ADSR {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
}

impl Envelope for ADSR {
    fn amplitude(&self, t: f64, note_duration: f64) -> f64 {
        let release_start = note_duration;
        let end_time = release_start + self.release;

        if t < self.attack {
            t / self.attack
        } else if t < self.attack + self.decay {
            let decay_t = t - self.attack;
            1.0 - (1.0 - self.sustain) * (decay_t / self.decay)
        } else if t < release_start {
            self.sustain
        } else if t < end_time {
            let release_t = t - release_start;
            self.sustain * (1.0 - release_t / self.release)
        } else {
            0.0
        }
    }
}

pub struct RenderState {
    pub time_beats: f64,
    pub tempo_bpm: f64,
    pub velocity: f64,
    pub divisions: u32,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            time_beats: 0.0,
            tempo_bpm: 120.0,
            velocity: 80.0 / 127.0, // mf
            divisions: 480,
        }
    }
}

impl RenderState {
    pub fn seconds_per_beat(&self) -> f64 {
        60.0 / self.tempo_bpm
    }

    pub fn advance(&mut self, duration_divs: u32) {
        let duration_beats = duration_divs as f64 / self.divisions as f64;
        self.time_beats += duration_beats;
    }

    pub fn rewind(&mut self, duration_divs: u32) {
        let duration_beats = duration_divs as f64 / self.divisions as f64;
        self.time_beats -= duration_beats;
    }
}

#[derive(Clone, Copy)]
pub enum OscillatorType {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl OscillatorType {
    pub fn sample(&self, freq: f64, t: f64) -> f64 {
        let phase = 2.0 * PI * freq * t;
        match self {
            Self::Sine => phase.sin(),
            Self::Saw => 2.0 * (t * freq - (t * freq + 0.5).floor()),
            Self::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Triangle => {
                2.0 * (2.0 * (t * freq - (t * freq + 0.5).floor())).abs() - 1.0
            }
        }
    }
}

// INSTRUMENTS

pub struct Synth {
    pub oscillator: OscillatorType,
    pub envelope: Box<dyn Envelope>,
    pub gain: f64,
    pub sample_rate: f64,
}

impl Instrument for Synth {
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<f64> {
        let mut buffer = vec![0.0; ctx.sample_rate * 60]; // buffer for 60s, resize later
        let mut state = RenderState::default();

        for measure in &part.measures {
            for item in &measure.items {
                match item {
                    MeasureItem::Note(note) => {
                        if let Some(pitch) = &note.pitch {
                            let freq = pitch.to_frequency();
                            let duration_beats =
                                note.duration as f64 / state.divisions as f64;
                            let duration_secs =
                                duration_beats * state.seconds_per_beat();
                            let start_sample = (state.time_beats
                                * state.seconds_per_beat()
                                * ctx.sample_rate as f64)
                                .round()
                                as usize;
                            let num_samples = (duration_secs
                                * ctx.sample_rate as f64)
                                .round()
                                as usize;

                            for i in 0..num_samples {
                                let t = i as f64 / ctx.sample_rate as f64;
                                let amp =
                                    self.envelope.amplitude(t, duration_secs);
                                let sample = self.oscillator.sample(freq, t)
                                    * amp
                                    * self.gain
                                    * state.velocity;
                                let idx = start_sample + i;
                                if idx < buffer.len() {
                                    buffer[idx] += sample;
                                }
                            }

                            state.advance(note.duration);
                        }
                    }

                    MeasureItem::Direction(dir) => match &dir.kind {
                        DirectionType::Metronome { per_minute, .. } => {
                            state.tempo_bpm = *per_minute as f64;
                        }
                        DirectionType::Dynamics(dynamics) => {
                            state.velocity = dynamics.normalized_velocity();
                        }
                        _ => {}
                    },

                    // TODO duration is measured in ticks. convert to time
                    MeasureItem::Forward(fwd) => {
                        state.advance(fwd.duration);
                    }

                    MeasureItem::Backup(bak) => {
                        state.rewind(bak.duration);
                    }
                }
            }
        }

        buffer
    }
}
