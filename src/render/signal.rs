/// Tools for generating and modulating signals
use std::cell::RefCell;
use std::f64::consts::PI;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use super::types::{Float, Hz, Seconds};

pub enum SignalSource {
    Oscillator(Oscillator),
    Noise(Noise),
    Sampler(Sampler),
    Silence,
}

impl SignalSource {
    /// Noise and silence ignore t
    pub fn sample(&self, t: Seconds) -> Float {
        match self {
            Self::Oscillator(osc) => osc.sample(t),
            Self::Noise(n) => n.sample(),
            Self::Sampler(s) => s.sample(t),
            Self::Silence => 0.0,
        }
    }
}

// TODO Implement
pub struct Sampler;

mod wave {
    use super::super::types::{DutyCycle, Float, SampleBuffer, Skew};
    type Dimensions = usize;

    pub struct Wavetable {
        pub tables: Vec<SampleBuffer>,

        /// Number of samples per waveform (assumes all are same length)
        pub resolution: usize,
        pub dimensions: usize,
    }

    pub struct WavetablePosition<const N: Dimensions> {
        /// One coord per dimension
        /// 1D: [x]
        /// 2D: [x, y]
        /// 3D: [x, y, x]
        pub coords: [Float; N], // One per dimension, range [0.0, 1.0]
    }

    pub enum WaveShape {
        Sine,
        Triangle(Skew),
        Saw(Skew),
        Pulse(DutyCycle),
    }

    /// Linearly interpolates between `a` and `b` by the interpolation factor
    /// `t`.
    ///
    /// - `t = 0.0` returns `a`
    /// - `t = 1.0` returns `b`
    /// - Values of `t` between 0.0 and 1.0 return a weighted average
    ///
    /// Useful for smooth transitions or blending between two values.
    fn lerp(a: Float, b: Float, t: Float) -> Float {
        a * (1.0 - t) + b * t
    }

    /// `phase`: in [0.0, 1.0), the position in the waveform
    /// `position.coords`: in [0.0, 1.0]
    pub fn interpolate_sample(
        wavetable: &Wavetable,
        position: &WavetablePosition<N>,
        phase: Float,
    ) -> Float {
        let resolution = wavetable.resolution;
        let sample_index = phase * (resolution as Float);
        let i0 = sample_index.floor() as usize % resolution;
        let i1 = (i0 + 1) % resolution;
        let frac = sample_index.fract();

        match wavetable.dimensions {
            1 => {
                // Linear morph between 2 tables
                let t = position.coords[0].clamp(0.0, 1.0);
                let max_idx = wavetable.tables.len() - 1;
                let idx = (t * max_idx as Float).floor() as usize;
                let next_idx = (idx + 1).min(max_idx);
                let local_t = (t * max_idx as Float) - idx as Float;

                let a = lerp(
                    wavetable.tables[idx][i0],
                    wavetable.tables[idx][i1],
                    frac,
                );
                let b = lerp(
                    wavetable.tables[next_idx][i0],
                    wavetable.tables[next_idx][i1],
                    frac,
                );
                lerp(a, b, local_t)
            }

            2 => {
                // Bilinear morph between 4 tables
                let x = position.coords[0].clamp(0.0, 1.0);
                let y = position.coords[1].clamp(0.0, 1.0);
                let grid_w = (wavetable.tables.len() as f64).sqrt() as usize;
                let grid_h = grid_w;

                let x_idx = (x * (grid_w - 1) as Float).floor() as usize;
                let y_idx = (y * (grid_h - 1) as Float).floor() as usize;
                let tx = x * (grid_w - 1) as Float - x_idx as Float;
                let ty = y * (grid_h - 1) as Float - y_idx as Float;

                let idx = |x: usize, y: usize| y * grid_w + x;

                let a = lerp(
                    wavetable.tables[idx(x_idx, y_idx)][i0],
                    wavetable.tables[idx(x_idx, y_idx)][i1],
                    frac,
                );
                let b = lerp(
                    wavetable.tables[idx(x_idx + 1, y_idx)][i0],
                    wavetable.tables[idx(x_idx + 1, y_idx)][i1],
                    frac,
                );
                let c = lerp(
                    wavetable.tables[idx(x_idx, y_idx + 1)][i0],
                    wavetable.tables[idx(x_idx, y_idx + 1)][i1],
                    frac,
                );
                let d = lerp(
                    wavetable.tables[idx(x_idx + 1, y_idx + 1)][i0],
                    wavetable.tables[idx(x_idx + 1, y_idx + 1)][i1],
                    frac,
                );

                let top = lerp(a, b, tx);
                let bottom = lerp(c, d, tx);
                lerp(top, bottom, ty)
            }

            3 => {
                // Trilinear morph between 8 tables
                let x = position.coords[0].clamp(0.0, 1.0);
                let y = position.coords[1].clamp(0.0, 1.0);
                let z = position.coords[2].clamp(0.0, 1.0);

                let grid_size = (wavetable.tables.len() as f64).cbrt() as usize;
                let idx = |x: usize, y: usize, z: usize| {
                    z * grid_size * grid_size + y * grid_size + x
                };

                let x0 = (x * (grid_size - 1) as Float).floor() as usize;
                let y0 = (y * (grid_size - 1) as Float).floor() as usize;
                let z0 = (z * (grid_size - 1) as Float).floor() as usize;
                let tx = x * (grid_size - 1) as Float - x0 as Float;
                let ty = y * (grid_size - 1) as Float - y0 as Float;
                let tz = z * (grid_size - 1) as Float - z0 as Float;

                let sample = |x, y, z| {
                    let buf = &wavetable.tables[idx(x, y, z)];
                    lerp(buf[i0], buf[i1], frac)
                };

                let c000 = sample(x0, y0, z0);
                let c100 = sample(x0 + 1, y0, z0);
                let c010 = sample(x0, y0 + 1, z0);
                let c110 = sample(x0 + 1, y0 + 1, z0);
                let c001 = sample(x0, y0, z0 + 1);
                let c101 = sample(x0 + 1, y0, z0 + 1);
                let c011 = sample(x0, y0 + 1, z0 + 1);
                let c111 = sample(x0 + 1, y0 + 1, z0 + 1);

                let c00 = lerp(c000, c100, tx);
                let c10 = lerp(c010, c110, tx);
                let c01 = lerp(c001, c101, tx);
                let c11 = lerp(c011, c111, tx);

                let c0 = lerp(c00, c10, ty);
                let c1 = lerp(c01, c11, ty);

                lerp(c0, c1, tz)
            }

            _ => panic!("Unsupported wavetable dimension"),
        }
    }
}

pub struct Oscillator {
    pub table: Wavetable,
    pub position: WavetablePosition,
    pub freq: Hz,
    pub phase: Float,
}

impl Default for Oscillator {
    fn default() -> Self {
        Self { wave: WaveShape::Sine, freq: 440.0 }
    }
}

impl Oscillator {
    /// Return sample of waveform at specified frequency (f) and time (t)
    pub fn sample(&self, t: Seconds) -> Float {
        let f = self.freq;
        let phase = 2.0 * PI * f * t;

        match self.wave {
            WaveShape::Sine => phase.sin(),
            WaveShape::Saw(skew) => {
                if skew == 0.5 {
                    // Classic symmetric sawtooth: rising from -1 to 1
                    2.0 * (phase - 0.5)
                } else if skew < 0.5 {
                    let norm_phase = phase / (2.0 * skew);
                    if phase < skew {
                        2.0 * norm_phase - 1.0
                    } else {
                        1.0
                    }
                } else {
                    let norm_phase = (phase - skew) / (1.0 - skew);
                    if phase >= skew {
                        -2.0 * norm_phase + 1.0
                    } else {
                        -1.0
                    }
                }
            }
            WaveShape::Triangle(skew) => {
                if skew == 0.5 {
                    // Classic triangle
                    4.0 * (phase - 0.5).abs() - 1.0
                } else if phase < skew {
                    // Rising segment
                    (2.0 / skew) * phase - 1.0
                } else {
                    // Falling segment
                    (-2.0 / (1.0 - skew)) * (phase - skew) + 1.0
                }
            }
            WaveShape::Pulse(duty) => {
                let phase_fraction = (f * t / (2.0 * PI)) % 1.0;
                if phase_fraction < duty {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }
}

pub struct Noise {
    pub kind: NoiseType,
    pub rng: RefCell<StdRng>,

    // TODO used for brown noise. refactor probably needed
    pub last_sample: RefCell<Float>,
}

pub enum NoiseType {
    White,
    Brown,
    // TODO add more noise types
}

impl Noise {
    pub fn new(kind: NoiseType, seed: u64) -> Self {
        Self {
            kind,
            rng: RefCell::new(StdRng::seed_from_u64(seed)),
            last_sample: RefCell::new(0.0),
        }
    }
}

impl Noise {
    pub fn sample(&self) -> Float {
        let mut rng = self.rng.borrow_mut();

        match self.kind {
            NoiseType::White => rng.gen_range(-1.0..1.0),
            NoiseType::Brown => {
                let mut last = self.last_sample.borrow_mut();
                let next =
                    (*last + rng.gen_range(-0.02..0.02)).clamp(-1.0, 1.0);
                *last = next;
                next
            }
        }
    }
}

pub enum ModulationSource {
    Constant(Float),
    Envelope(ParametricEnvelope),
    LowFrequencyOscillator(Oscillator),
}

impl ModulationSource {
    fn value_at(&self, t: Seconds) -> Float {
        match self {
            Self::Constant(v) => v,
            Self::Envelope(e) => e.value(t),
            Self::LowFrequencyOscillator(osc) => osc.sample(t),
        }
    }
}

pub enum ModulationTarget {
    // Oscillator / note
    Pitch,
    Amplitude,
    Velocity,
    DutyCycle,
    PhaseOffset,

    // Effects
    FilterCutoff,
    FilterResonance,
    DistortionDrive,
    PanPosition,
    ReverbMix,
    DelayTime,
    DelayFeedback,
    ChorusDepth,
    BitCrushAmount,

    // Envelopes
    AttackTime,
    DecayTime,
    SustainLevel,
    ReleaseTime,

    // Modulations
    ModulationDepth,
}

pub struct ModulationRoute {
    pub source: ModulationSource,
    pub target: ModulationTarget,
    pub depth: Float, // -1.0..1.0
}

pub struct ModulationMatrix {
    pub routes: Vec<ModulationRoute>,
}

impl ModulationMatrix {
    pub fn apply(&mut self, t: Float) {
        for r in &mut self.routes {
            let val = r.source.sample(t) * r.depth;
            r.target.apply(val);
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvelopeStage {
    pub kind: StageKind,
    pub start_level: Float,
    pub end_level: Float,
}

enum StageKind {
    /// A ramp with a time duration and curvature.
    /// curve: 1.0 = linear, >1.0 = exponential, <1.0 = logarithmic
    Ramp { duration: Float, curve: Float },

    /// Sustain level; this stage is held until the gate is released.
    Sustain,

    /// Hold a value for a fixed time (no ramp).
    Hold { duration: Float },

    /// Step to a new level immediately.
    Step,
}

#[derive(Debug, Clone)]
pub struct ParametricEnvelope {
    pub stages: Vec<EnvelopeStage>,
    pub loop_point: Option<usize>, /* optional: loop stage N to end until
                                    * note off */
    pub sustain_point: Option<usize>, // optional: hold here until note off
    pub release_stages: Vec<EnvelopeStage>,
    pub gate_on_time: Option<Float>,
    pub gate_off_time: Option<Float>,
}

impl ParametricEnvelope {
    pub fn new(
        stages: Vec<EnvelopeStage>,
        release_stages: Vec<EnvelopeStage>,
    ) -> Self {
        Self {
            stages,
            loop_point: None,
            sustain_point: None,
            release_stages,
            gate_on_time: None,
            gate_off_time: None,
        }
    }

    pub fn gate_on(&mut self, time: Float) {
        self.gate_on_time = Some(time);
        self.gate_off_time = None;
    }

    pub fn gate_off(&mut self, time: Float) {
        self.gate_off_time = Some(time);
    }

    pub fn value(&self, now: Float) -> Float {
        let gate_on = match self.gate_on_time {
            Some(t) => t,
            None => return 0.0,
        };

        let t = now - gate_on;

        // If note is still on
        if self.gate_off_time.is_none() {
            let mut cursor = 0.0;
            for (i, stage) in self.stages.iter().enumerate() {
                match &stage.kind {
                    StageKind::Ramp { duration, curve } => {
                        if t < cursor + duration {
                            let local_t = t - cursor;
                            let x = local_t / duration;
                            let shaped = (1.0 - x).powf(*curve);
                            return stage.end_level
                                + (stage.start_level - stage.end_level)
                                    * shaped;
                        }
                        cursor += duration;
                    }
                    StageKind::Hold { duration } => {
                        if t < cursor + duration {
                            return stage.start_level;
                        }
                        cursor += duration;
                    }
                    StageKind::Step => {
                        if t < cursor + 0.001 {
                            return stage.end_level;
                        }
                        cursor += 0.001;
                    }
                    StageKind::Sustain => {
                        if let Some(sustain_idx) = self.sustain_point {
                            if i == sustain_idx {
                                return stage.start_level;
                            }
                        }
                        cursor += 0.0; // Sustain is zero-time (held)
                    }
                }
            }

            return self.stages.last().map(|s| s.end_level).unwrap_or(0.0);
        }

        // Note is off, proceed through release stages
        let release_t = now - self.gate_off_time.unwrap();
        let mut cursor = 0.0;
        for stage in &self.release_stages {
            match &stage.kind {
                StageKind::Ramp { duration, curve } => {
                    if release_t < cursor + duration {
                        let local_t = release_t - cursor;
                        let x = local_t / duration;
                        let shaped = (1.0 - x).powf(*curve);
                        return stage.end_level
                            + (stage.start_level - stage.end_level) * shaped;
                    }
                    cursor += duration;
                }
                StageKind::Hold { duration } => {
                    if release_t < cursor + duration {
                        return stage.start_level;
                    }
                    cursor += duration;
                }
                StageKind::Step => {
                    if release_t < cursor + 0.001 {
                        return stage.end_level;
                    }
                    cursor += 0.001;
                }
                StageKind::Sustain => {
                    unreachable!("Sustain not allowed in release phase")
                }
            }
        }

        0.0 // End of release phase
    }

    pub fn release_time(&self) -> Float {
        self.release_stages
            .iter()
            .map(|s| match &s.kind {
                StageKind::Ramp { duration, .. } => *duration,
                StageKind::Hold { duration } => *duration,
                StageKind::Step => 0.001,
                StageKind::Sustain => 0.0,
            })
            .sum()
    }

    /// Convenience for standard ADSR
    pub fn from_adsr(
        attack: Float,
        decay: Float,
        sustain: Float,
        release: Float,
    ) -> Self {
        Self {
            stages: vec![
                EnvelopeStage {
                    kind: StageKind::Ramp { duration: attack, curve: 1.0 },
                    start_level: 0.0,
                    end_level: 1.0,
                },
                EnvelopeStage {
                    kind: StageKind::Ramp { duration: decay, curve: 1.0 },
                    start_level: 1.0,
                    end_level: sustain,
                },
                EnvelopeStage {
                    kind: StageKind::Sustain,
                    start_level: sustain,
                    end_level: sustain,
                },
            ],
            release_stages: vec![EnvelopeStage {
                kind: StageKind::Ramp { duration: release, curve: 1.0 },
                start_level: sustain,
                end_level: 0.0,
            }],
            sustain_point: Some(2),
            loop_point: None,
            gate_on_time: None,
            gate_off_time: None,
        }
    }

    /// Convenience for parametric decay envelope
    pub fn from_decay(
        start: Float,
        end: Float,
        duration: Float,
        curve: Float,
    ) -> Self {
        Self {
            stages: vec![EnvelopeStage {
                kind: StageKind::Ramp { duration, curve },
                start_level: start,
                end_level: end,
            }],
            release_stages: vec![],
            loop_point: None,
            sustain_point: None,
            gate_on_time: None,
            gate_off_time: None,
        }
    }
}
