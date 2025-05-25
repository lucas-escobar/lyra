use crate::compose::{DirectionType, MeasureItem, Part};
use std::f64::consts::PI;

pub struct RenderContext {
    pub sample_rate: usize,
}

pub trait Instrument {
    /// Renders a MusicXML part
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<f64>;
}

pub trait Envelope: Send + Sync {
    fn amplitude(&self, t: f64, note_duration: f64) -> f64;

    /// The release time of the envelope will extend the notated duration of
    /// a given note. This function provides this extra time to the renderer
    /// for proper duration calculations.
    fn release_time(&self) -> f64;
}

/// Duration represented in seconds.
pub struct ADSR {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64, // 0.0 - 1.0 value representing % of max volume
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

    fn release_time(&self) -> f64 {
        self.release
    }
}

pub struct RenderState {
    pub time_beats: f64,
    pub tempo_bpm: f64,
    pub velocity: f64,
    pub divisions: u32,
    pub active_voices: Vec<Voice>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            time_beats: 0.0,
            tempo_bpm: 120.0,
            velocity: 80.0 / 127.0, // mf
            divisions: 480,
            active_voices: vec![],
        }
    }
}

impl RenderState {
    pub fn seconds_per_beat(&self) -> f64 {
        60.0 / self.tempo_bpm
    }

    pub fn time_secs(&self) -> f64 {
        self.time_beats * self.seconds_per_beat()
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

pub struct Voice {
    pub freq: f64,
    pub start_time_secs: f64,
    pub duration_secs: f64,
    pub velocity: f64,
}

// INSTRUMENTS

pub struct Synth {
    pub oscillator: OscillatorType,
    pub envelope: Box<dyn Envelope>,
}

impl Instrument for Synth {
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<f64> {
        let mut state = RenderState::default();

        for measure in &part.measures {
            let mut last_start_time_beats = state.time_beats;
            for item in &measure.items {
                match item {
                    MeasureItem::Note(note) => {
                        if !note.is_chord {
                            // Cache the time cursor for chord use
                            last_start_time_beats = state.time_beats;
                        }

                        if let Some(pitch) = &note.pitch {
                            let freq = pitch.to_frequency();
                            let duration_beats =
                                note.duration as f64 / state.divisions as f64;
                            let duration_secs =
                                duration_beats * state.seconds_per_beat();
                            let start_time_secs = last_start_time_beats
                                * state.seconds_per_beat();

                            // Voices in this context are essentially
                            // synth events. There is one voice per each
                            // note processed. This is inefficient but
                            // good enough for now
                            let voice = Voice {
                                freq,
                                start_time_secs,
                                duration_secs,
                                velocity: state.velocity,
                            };

                            state.active_voices.push(voice);
                        }

                        if !note.is_chord {
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

        // === Render all voices into buffer with voice-aware scaling ===
        let buffer_len = ctx.sample_rate * 60;
        let mut mix_buffer: Vec<f64> = vec![0.0; buffer_len];
        let mut voice_count: Vec<f64> = vec![0.0; buffer_len];

        for voice in &state.active_voices {
            let full_duration_secs =
                voice.duration_secs + self.envelope.release_time();
            let start_sample = (voice.start_time_secs * ctx.sample_rate as f64)
                .round() as usize;
            let num_samples =
                (full_duration_secs * ctx.sample_rate as f64).round() as usize;

            for i in 0..num_samples {
                let t = i as f64 / ctx.sample_rate as f64;
                let amp = self.envelope.amplitude(t, voice.duration_secs);
                let sample = self.oscillator.sample(voice.freq, t)
                    * amp
                    * voice.velocity;
                let idx = start_sample + i;
                if idx < buffer_len {
                    mix_buffer[idx] += sample;

                    // Only count this voice as "active" if it hasn't entered release phase
                    if t < voice.duration_secs {
                        voice_count[idx] += 1.0;
                    }
                }
            }
        }

        // Scale loudness
        let mut final_buffer = vec![0.0; buffer_len];
        let mut smoothed_gain = 1.0;
        let alpha = 0.01;
        let mut gain_frozen = false;

        for i in 0..buffer_len {
            let raw_gain = if voice_count[i] > 0.0 {
                gain_frozen = false;
                1.0 / (voice_count[i] + 1.0).powf(0.65)
            } else if !gain_frozen {
                gain_frozen = true;
                smoothed_gain // Freeze at current level
            } else {
                smoothed_gain // Keep using last smoothed value
            };

            smoothed_gain = alpha * raw_gain + (1.0 - alpha) * smoothed_gain;
            final_buffer[i] = mix_buffer[i] * smoothed_gain;
        }

        final_buffer
    }
}
