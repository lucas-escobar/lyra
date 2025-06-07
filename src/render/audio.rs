use crate::compose::{DirectionType, MeasureItem, Part, Pitch, StartStop};
use std::collections::HashMap;
use std::f64::consts::PI;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

/// Sample buffers in the render layer have double float precision
type Float = f64;

pub trait UnpitchedInstrument {
    fn synth(&self, t: Float, dur: Float, state: &RenderState) -> Float;

    /// Renders a MusicXML part
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<Float> {
        let mut output = Vec::new();
        let mut state = RenderState::default();
        let sr = ctx.sample_rate;

        for m in &part.measures {
            for item in &m.items {
                match item {
                    MeasureItem::Note(note) => {
                        if note.unpitched.is_some() {
                            let dur = state.ticks_to_secs(note.duration);
                            let n = state.ticks_to_samples(note.duration, sr);
                            let start_sample = (state.time_secs() * sr as Float)
                                .floor()
                                as usize;

                            if output.len() < start_sample + n {
                                output.resize(start_sample + n, 0.0);
                            }

                            for i in 0..n {
                                let t = i as Float / sr as Float;
                                let sample = self.synth(t, dur, &state);
                                output[start_sample + i] += sample;
                            }
                        }
                        state.advance(note.duration);
                    }

                    MeasureItem::Direction(dir) => match &dir.kind {
                        DirectionType::Metronome { per_minute, .. } => {
                            state.tempo_bpm = *per_minute as Float;
                        }
                        DirectionType::Dynamics(dynamics) => {
                            state.velocity = dynamics.normalized_velocity();
                        }
                        _ => {}
                    },

                    MeasureItem::Barline(_) => {}

                    _ => {}
                }
            }
        }

        output
    }
}

pub trait Envelope: Send + Sync {
    /// Returns a value from the envelope based on current time.
    fn value(&self, t: Float, duration: Float) -> Float;

    /// If the user needs to know how long the release will last.
    fn release_time(&self) -> Float;
}

pub struct ParametricDecayEnvelope {
    pub start: Float,
    pub end: Float,
    pub exponent: Float, // 1.0 = linear, >1 = exponential, <1 = log-like
}

impl Envelope for ParametricDecayEnvelope {
    fn value(&self, t: Float, duration: Float) -> Float {
        assert!(
            self.start != self.end,
            "Start and end values of envelope cannot
            be equal"
        );
        assert!(
            self.end < self.start,
            "Start time has to be before end time in decay envelope"
        );

        if t >= duration {
            return self.end;
        }

        let progress = t / duration;
        let shaped = (1.0 - progress).powf(self.exponent);
        let start = self.start;
        let end = self.end;
        end + (start - end) * shaped
    }

    fn release_time(&self) -> Float {
        0.0
    }
}

impl ParametricDecayEnvelope {
    fn reaches_threshold(&self, duration: Float, threshold: Float) -> bool {
        let end_val = self.value(duration, duration);
        (end_val - self.end).abs() <= threshold
    }
}

/// Duration represented in seconds.
pub struct ADSR {
    pub attack: Float,
    pub decay: Float,
    pub sustain: Float, // 0.0 - 1.0 value representing % of max volume
    pub release: Float,
}

impl Envelope for ADSR {
    fn value(&self, t: Float, note_duration: Float) -> Float {
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

    fn release_time(&self) -> Float {
        self.release
    }
}

impl ADSR {
    /// The release time of the envelope will extend the notated duration of
    /// a given note. This function provides this extra time to the renderer
    /// for proper duration calculations.
    fn release_time(&self) -> Float {
        self.release
    }
}

pub struct RenderContext {
    pub sample_rate: u32,
}

pub struct RenderState {
    pub time_beats: Float,
    pub tempo_bpm: Float,
    pub velocity: Float,
    pub divisions: u32,
    pub active_voices: Vec<Voice>,
    // Pitch => (start_time_beats, total_duration_beats)
    pub ongoing_ties: HashMap<u8, (Float, Float)>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            time_beats: 0.0,
            tempo_bpm: 120.0,
            velocity: 80.0 / 127.0, // mf
            divisions: 480,
            active_voices: vec![],
            ongoing_ties: HashMap::new(),
        }
    }
}

impl RenderState {
    pub fn seconds_per_beat(&self) -> Float {
        60.0 / self.tempo_bpm
    }

    pub fn time_secs(&self) -> Float {
        self.time_beats * self.seconds_per_beat()
    }

    /// Convert a duration in ticks to a duration in samples
    pub fn ticks_to_samples(&self, duration: u32, sample_rate: u32) -> usize {
        let dur_beats = duration as Float / self.divisions as Float;
        let dur_sec = dur_beats * self.seconds_per_beat();
        (dur_sec * sample_rate as Float).ceil() as usize
    }

    /// Convert a duration in ticks to a duration in samples
    pub fn ticks_to_secs(&self, duration: u32) -> Float {
        let dur_beats = duration as Float / self.divisions as Float;
        dur_beats * self.seconds_per_beat()
    }

    pub fn advance(&mut self, duration_divs: u32) {
        let duration_beats = duration_divs as Float / self.divisions as Float;
        self.time_beats += duration_beats;
    }

    pub fn rewind(&mut self, duration_divs: u32) {
        let duration_beats = duration_divs as Float / self.divisions as Float;
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
    pub fn sample(&self, freq: Float, t: Float) -> Float {
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
    pub freq: Float,
    pub start_time_secs: Float,
    pub duration_secs: Float,
    pub velocity: Float,
}

// INSTRUMENTS

pub struct Synth {
    pub oscillator: OscillatorType,
    // TODO make envelope choice more flexible
    pub envelope: ADSR,
}

impl Synth {
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<Float> {
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
                            let midi = pitch.to_semitone();
                            let duration_beats = note.duration as Float
                                / state.divisions as Float;
                            let duration_secs =
                                duration_beats * state.seconds_per_beat();
                            let start_time_secs = last_start_time_beats
                                * state.seconds_per_beat();

                            match note.tie {
                                Some(StartStop::Start) => {
                                    state.ongoing_ties.insert(
                                        midi,
                                        (start_time_secs, duration_secs),
                                    );
                                }
                                Some(StartStop::Stop) => {
                                    if let Some((prev_start, prev_duration)) =
                                        state.ongoing_ties.remove(&midi)
                                    {
                                        let combined_duration =
                                            prev_duration + duration_secs;
                                        let voice = Voice {
                                            freq,
                                            start_time_secs: prev_start,
                                            duration_secs: combined_duration,
                                            velocity: state.velocity,
                                        };
                                        state.active_voices.push(voice);
                                    } else {
                                        // Treat it as a normal note if we have no matching start
                                        let voice = Voice {
                                            freq,
                                            start_time_secs,
                                            duration_secs,
                                            velocity: state.velocity,
                                        };
                                        state.active_voices.push(voice);
                                    }
                                }
                                _ => {
                                    // Not part of a tie, render normally
                                    let voice = Voice {
                                        freq,
                                        start_time_secs,
                                        duration_secs,
                                        velocity: state.velocity,
                                    };
                                    state.active_voices.push(voice);
                                }
                            }
                        }

                        if !note.is_chord {
                            state.advance(note.duration);
                        }
                    }

                    MeasureItem::Direction(dir) => match &dir.kind {
                        DirectionType::Metronome { per_minute, .. } => {
                            state.tempo_bpm = *per_minute as Float;
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

                    MeasureItem::Barline(_) => {}
                }
            }
        }

        // TODO calculate precice buffer len
        let buffer_len: usize = (ctx.sample_rate * 60) as usize;

        // TODO should these be fixed sized arrays?
        let mut mix_buffer: Vec<Float> = vec![0.0; buffer_len];
        let mut loudness_sum: Vec<Float> = vec![0.0; buffer_len];

        for voice in &state.active_voices {
            let full_duration_secs =
                voice.duration_secs + self.envelope.release_time();
            let start_sample = (voice.start_time_secs
                * ctx.sample_rate as Float)
                .round() as usize;
            let num_samples = (full_duration_secs * ctx.sample_rate as Float)
                .round() as usize;

            for i in 0..num_samples {
                let t = i as Float / ctx.sample_rate as Float;
                let amp = self.envelope.value(t, voice.duration_secs);
                let sample = self.oscillator.sample(voice.freq, t)
                    * amp
                    * voice.velocity;
                let idx = start_sample + i;

                if idx < buffer_len {
                    mix_buffer[idx] += sample;
                    loudness_sum[idx] += amp * voice.velocity;
                }
            }
        }

        let mut final_buffer = vec![0.0; buffer_len];
        let mut smoothed_gain = 1.0;
        let alpha = 0.01;
        let mut gain_frozen = false;

        for i in 0..buffer_len {
            let raw_gain = if loudness_sum[i] > 0.0 {
                gain_frozen = false;
                1.0 / (loudness_sum[i] + 1.0).powf(0.8)
            } else if !gain_frozen {
                gain_frozen = true;
                smoothed_gain
            } else {
                smoothed_gain
            };

            smoothed_gain = alpha * raw_gain + (1.0 - alpha) * smoothed_gain;
            final_buffer[i] = (mix_buffer[i] * smoothed_gain).clamp(-1.0, 1.0);
        }

        final_buffer
    }
}

pub struct Distortion {
    pub drive: Float,
}

impl Distortion {
    pub fn apply(&self, sample: Float) -> Float {
        // Soft clipping
        (sample * self.drive).tanh()
    }
}

pub struct Transient {
    pub duration: Float,
    pub amplitude: Float,
    pub freq: Float, // for sine or tone-based clicks
}

impl Transient {
    pub fn sample(&self, t: Float) -> Float {
        if t >= self.duration {
            0.0
        } else {
            (2.0 * PI * self.freq * t).sin() * self.amplitude
        }
    }
}

pub struct KickDrum {
    pub amp_env: ParametricDecayEnvelope,
    pub freq_env: ParametricDecayEnvelope,
    pub distortion: Option<Distortion>,
    pub transient: Option<Transient>,
}

impl UnpitchedInstrument for KickDrum {
    fn synth(&self, t: Float, dur: Float, state: &RenderState) -> Float {
        let amp = self.amp_env.value(t, dur);
        let freq = self.freq_env.value(t, dur);
        let mut sample =
            OscillatorType::Sine.sample(freq, t) * amp * state.velocity;
        if let Some(d) = &self.distortion {
            sample = d.apply(sample)
        };
        sample
    }
}

pub struct SnareDrum {
    pub amp_env: ParametricDecayEnvelope,
    pub noise_env: ParametricDecayEnvelope,
    //let mut rng = StdRng::seed_from_u64(42); // deterministic noise
    pub rng: StdRng,
    pub tone_env: Option<ParametricDecayEnvelope>,
    pub freq: Option<Float>, // optional tonal frequency
    pub distortion: Option<Distortion>,
    pub transient: Option<Transient>,
}

impl UnpitchedInstrument for SnareDrum {
    fn synth(&self, t: Float, dur: Float, state: &RenderState) -> Float {
        let amp = self.amp_env.value(t, dur);
        let noise_amp = self.noise_env.value(t, dur);
        let noise_sample: Float = self.rng.random_range(-1.0..1.0);

        let tonal_sample =
            if let (Some(freq), Some(env)) = (self.freq, &self.tone_env) {
                let tone_amp = env.value(t, dur);
                (2.0 * PI * freq * t).sin() * tone_amp
            } else {
                0.0
            };

        let mut sample = (noise_sample * noise_amp) + tonal_sample;
        sample *= amp * state.velocity;

        if let Some(d) = &self.distortion {
            sample = d.apply(sample)
        };
        sample
    }
}

pub struct HiHat {
    pub amp_env: ParametricDecayEnvelope,
    pub rng: StdRng,
}

impl UnpitchedInstrument for HiHat {
    fn synth(&self, t: Float, dur: Float, state: &RenderState) -> Float {
        let amp = self.amp_env.value(t, dur);
        let noise = self.rng.random_range(-1.0..1.0); // white noise
        let sample = noise * amp * state.velocity;
        sample
    }
}
