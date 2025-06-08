// TODO remove the compose dependencies by making intermediate representation
// of Part
use crate::compose::{DirectionType, MeasureItem, Part, StartStop};

use super::envelope::{Envelope, ParametricDecayEnvelope, ADSR};
use super::types::Float;

use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub trait Instrument: 'static {
    /// Render a MusicXML part into a sample buffer. This is the main interface
    /// between the compose layer and render layer.
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<Float>;
}

pub enum AnyInstrument {
    Pitched(Box<dyn PitchedInstrument>),
    Unpitched(Box<dyn UnpitchedInstrument>),
}

impl Instrument for AnyInstrument {
    fn render_part(&self, part: &Part, ctx: &RenderContext) -> Vec<Float> {
        match self {
            AnyInstrument::Pitched(inst) => inst.render_part(part, ctx),
            AnyInstrument::Unpitched(inst) => inst.render_part(part, ctx),
        }
    }
}

impl AnyInstrument {
    pub fn pitched<T: PitchedInstrument + 'static>(inst: T) -> Self {
        AnyInstrument::Pitched(Box::new(inst))
    }

    pub fn unpitched<T: UnpitchedInstrument + 'static>(inst: T) -> Self {
        AnyInstrument::Unpitched(Box::new(inst))
    }
}

pub trait PitchedInstrument {
    fn process_voices(
        &self,
        ctx: &RenderContext,
        voices: Vec<Voice>,
    ) -> Vec<Float>;

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

        self.process_voices(ctx, state.active_voices)
    }
}

pub trait UnpitchedInstrument {
    fn synth(&self, t: Float, dur: Float, state: &mut RenderState) -> Float;

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
                                let sample = self.synth(t, dur, &mut state);
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

/// Mutable state used during the rendering process
pub struct RenderState {
    pub time_beats: Float,
    pub tempo_bpm: Float,
    pub velocity: Float,
    pub divisions: u32,
    pub active_voices: Vec<Voice>,
    // Pitch => (start_time_beats, total_duration_beats)
    pub ongoing_ties: HashMap<u8, (Float, Float)>,
    pub rng: StdRng,
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
            rng: StdRng::seed_from_u64(1337),
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

// TODO pitched instruments collect all of the notes found in a part into voices.
// One note is assigned to each voice. This in inefficient; check if this matters.
pub struct Voice {
    pub freq: Float,
    pub start_time_secs: Float,
    pub duration_secs: Float,
    pub velocity: Float,
}

// CONCRETE INSTRUMENTS

pub struct Synth {
    pub oscillator: OscillatorType,
    // TODO make envelope choice more flexible
    pub envelope: ADSR,
}

impl Synth {
    pub fn process_voices(
        &self,
        ctx: &RenderContext,
        state: &mut RenderState,
    ) -> Vec<Float> {
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

impl Default for KickDrum {
    fn default() -> Self {
        Self {
            amp_env: ParametricDecayEnvelope {
                start: 1.0,
                end: 0.0,
                exponent: 3.0, // Fast exponential decay
            },
            freq_env: ParametricDecayEnvelope {
                start: 120.0,  // Starts high for transient punch
                end: 30.0,     // Drops to bass
                exponent: 5.0, // Sharp downward pitch drop
            },
            distortion: Some(Distortion { drive: 1.2 }), // Adds grit
            transient: None, // Optional: You can later add a snappy click here
        }
    }
}

impl UnpitchedInstrument for KickDrum {
    fn synth(&self, t: Float, dur: Float, state: &mut RenderState) -> Float {
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
    pub tone_env: Option<ParametricDecayEnvelope>,
    pub freq: Option<Float>, // optional tonal frequency
    pub distortion: Option<Distortion>,
    pub transient: Option<Transient>,
}

impl Default for SnareDrum {
    fn default() -> Self {
        Self {
            amp_env: ParametricDecayEnvelope {
                start: 1.0,
                end: 0.0,
                exponent: 3.0,
            },
            noise_env: ParametricDecayEnvelope {
                start: 1.0,
                end: 0.0,
                exponent: 3.0,
            },
            tone_env: Some(ParametricDecayEnvelope {
                start: 1.0,
                end: 0.0,
                exponent: 6.0,
            }),
            freq: Some(180.0), // optional tonal body
            distortion: Some(Distortion { drive: 3.0 }),
            transient: None,
        }
    }
}

impl UnpitchedInstrument for SnareDrum {
    fn synth(&self, t: Float, dur: Float, state: &mut RenderState) -> Float {
        let amp = self.amp_env.value(t, dur);
        let noise_amp = self.noise_env.value(t, dur);
        let noise_sample: Float = state.rng.random_range(-1.0..1.0);

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
}

impl Default for HiHat {
    fn default() -> Self {
        Self {
            amp_env: ParametricDecayEnvelope {
                start: 1.0,
                end: 0.0,
                exponent: 8.0,
            },
        }
    }
}

impl UnpitchedInstrument for HiHat {
    fn synth(&self, t: Float, dur: Float, state: &mut RenderState) -> Float {
        let amp = self.amp_env.value(t, dur);
        let noise = state.rng.random_range(-1.0..1.0); // white noise
        let sample = noise * amp * state.velocity;
        sample
    }
}
