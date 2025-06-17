/// Tools for generating and modulating signals
use super::types::{Float, Seconds};

pub mod signal {
    use std::cell::RefCell;

    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::SeedableRng;

    use super::super::types::{Float, Hz, Seconds};
    use super::wave;

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
                //Self::Sampler(s) => s.sample(t),
                Self::Silence => 0.0,
                _ => panic!("Not implemented"),
            }
        }

        pub fn set_frequency(&mut self, freq: Float) {
            if let SignalSource::Oscillator(ref mut osc) = self {
                osc.freq = freq;
            }
        }
    }

    // TODO Implement
    pub struct Sampler;

    pub struct Oscillator {
        pub wave: wave::Wave,
        pub freq: Hz,
        pub phase: Float,
    }

    impl Default for Oscillator {
        fn default() -> Self {
            Self { wave: wave::Wave::default(), freq: 440.0, phase: 0.0 }
        }
    }

    impl Oscillator {
        /// Return sample of waveform at specified frequency (f) and time (t)
        pub fn sample(&self, t: Seconds) -> Float {
            self.wave.sample(self.freq, t)
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
            match self.kind {
                NoiseType::White => {
                    self.rng.borrow_mut().random_range(-1.0..1.0)
                }
                NoiseType::Brown => {
                    let mut last = self.last_sample.borrow_mut();
                    let next = (*last
                        + self.rng.borrow_mut().random_range(-0.02..0.02))
                    .clamp(-1.0, 1.0);
                    *last = next;
                    next
                }
            }
        }
    }
}

pub mod wave {
    use std::f64::consts::PI;

    use super::super::types::{DutyCycle, Float, Hz, Seconds, Skew};

    pub struct Wave {
        pub source: Box<dyn WaveSource>,
        pub modifiers: Option<Vec<Box<dyn WaveModifier>>>,
    }

    impl Default for Wave {
        fn default() -> Self {
            Self { source: Box::new(WaveShape::Sine), modifiers: None }
        }
    }

    impl Wave {
        pub fn sample(&self, frequency: Hz, t: Seconds) -> Float {
            let mut s = self.source.sample(frequency, t);
            if let Some(modifiers) = &self.modifiers {
                for m in modifiers {
                    s = m.apply(s);
                }
            }
            s
        }
    }

    pub trait WaveModifier {
        fn apply(&self, sample: Float) -> Float;
    }

    pub trait WaveSource {
        fn sample(&self, frequency: Hz, t: Seconds) -> Float;
    }

    pub struct Wavetable1D {
        waves: Vec<Box<dyn WaveSource>>,
        position: Float,
    }

    impl WaveSource for Wavetable1D {
        fn sample(&self, frequency: Hz, t: Seconds) -> Float {
            let n = self.waves.len();
            if n == 0 {
                return 0.0;
            }

            let pos = self.position.clamp(0.0, 1.0);
            let idx = pos * (n - 1) as Float;
            let i = idx.floor() as usize;
            let frac = idx - i as Float;

            if i >= n - 1 {
                return self.waves[n - 1].sample(frequency, t);
            }

            let a = self.waves[i].sample(frequency, t);
            let b = self.waves[i + 1].sample(frequency, t);
            a * (1.0 - frac) + b * frac
        }
    }

    pub enum WaveShape {
        Sine,
        Triangle(Skew),
        Saw(Skew),
        Pulse(DutyCycle),
    }

    impl WaveSource for WaveShape {
        fn sample(&self, frequency: Hz, t: Seconds) -> Float {
            let phase = 2.0 * PI * frequency * t;
            match self {
                Self::Sine => phase.sin(),
                Self::Saw(skew) => {
                    if *skew == 0.5 {
                        // Classic symmetric sawtooth: rising from -1 to 1
                        2.0 * (phase - 0.5)
                    } else if *skew < 0.5 {
                        let norm_phase = phase / (2.0 * skew);
                        if phase < *skew {
                            2.0 * norm_phase - 1.0
                        } else {
                            1.0
                        }
                    } else {
                        let norm_phase = (phase - skew) / (1.0 - skew);
                        if phase >= *skew {
                            -2.0 * norm_phase + 1.0
                        } else {
                            -1.0
                        }
                    }
                }
                Self::Triangle(skew) => {
                    if *skew == 0.5 {
                        // Classic triangle
                        4.0 * (phase - 0.5).abs() - 1.0
                    } else if phase < *skew {
                        // Rising segment
                        (2.0 / skew) * phase - 1.0
                    } else {
                        // Falling segment
                        (-2.0 / (1.0 - skew)) * (phase - skew) + 1.0
                    }
                }
                Self::Pulse(duty) => {
                    let phase_fraction = (frequency * t / (2.0 * PI)) % 1.0;
                    if phase_fraction < *duty {
                        1.0
                    } else {
                        -1.0
                    }
                }
            }
        }
    }

    pub struct Drive(pub Float);
    impl WaveModifier for Drive {
        fn apply(&self, x: Float) -> Float {
            (x * self.0).tanh()
        }
    }

    pub enum ClipMode {
        Hard,
        Soft,
    }

    pub struct Clip {
        pub threshold: Float,
        pub mode: ClipMode,
    }

    impl WaveModifier for Clip {
        fn apply(&self, sample: Float) -> Float {
            match self.mode {
                ClipMode::Hard => sample.clamp(-self.threshold, self.threshold),
                ClipMode::Soft => {
                    let x_norm = sample / self.threshold;
                    self.threshold * x_norm.tanh()
                }
            }
        }
    }

    pub struct Fold {
        pub threshold: Float,
        pub mode: ClipMode,
    }

    impl WaveModifier for Fold {
        fn apply(&self, x: Float) -> Float {
            match self.mode {
                ClipMode::Hard => {
                    let t = self.threshold.abs();
                    if x > t {
                        t - (x - t)
                    } else if x < -t {
                        -t - (x + t)
                    } else {
                        x
                    }
                }
                ClipMode::Soft => {
                    // soft folding using sine-based wrapping for a smooth curve
                    let t = self.threshold.abs();
                    (t / PI) * (x / t * PI).sin()
                }
            }
        }
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
}

pub enum ModulationSource {
    Constant(Float),
    Envelope(ParametricEnvelope),
    LowFrequencyOscillator(signal::Oscillator),
}

impl ModulationSource {
    fn value_at(&self, t: Seconds) -> Float {
        match self {
            Self::Constant(v) => *v,
            Self::Envelope(e) => e.value(t),
            Self::LowFrequencyOscillator(osc) => osc.sample(t),
        }
    }
}

#[derive(PartialEq)]
// Assumes all modulation targets are represented by Float value
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

//impl ModulationTarget {
//    pub fn apply(&self, base: Float, mod_val: Float) -> Float {
//        match self {
//            Self::Pitch => base + mod_val,
//            Self::Amplitude => base * (1.0 + mod_val),
//            Self::Velocity => base * (1.0 + mod_val),
//            Self::FilterCutoff => base + mod_val,
//            _ => panic!("Modulation target not implemented"),
//        }
//    }
//}

pub enum ModulationMode {
    Add,
    Multiply,
}

pub struct ModulationRoute {
    pub source: ModulationSource,
    pub target: ModulationTarget,
    pub mode: ModulationMode,
    pub depth: Float, // 0.0..1.0
}

impl ModulationRoute {
    pub fn apply(&self, base: Float, t: Seconds) -> Float {
        let mod_val = self.source.value_at(t) * self.depth;
        match self.mode {
            ModulationMode::Add => base + mod_val,
            // TODO determine better way to control 0 centered multiplication
            ModulationMode::Multiply => base * mod_val,
        }
    }
}

pub struct ModulationMatrix {
    pub routes: Vec<ModulationRoute>,
}

impl ModulationMatrix {
    /// Apply modulation matrix to a value and return the modulated value
    pub fn apply(
        &self,
        target: ModulationTarget,
        base: Float,
        t: Seconds,
    ) -> Float {
        let mut combined = base;

        for route in &self.routes {
            if route.target == target {
                combined = route.apply(combined, t);
            }
        }

        combined
    }

    /// Turns all envelope gates in the matrix on
    pub fn gate_on(&mut self, t: Seconds) {
        for route in &mut self.routes {
            if let ModulationSource::Envelope(ref mut env) = route.source {
                env.gate_on(t);
            }
        }
    }

    /// Turns all envelope gates in the matrix off
    pub fn gate_off(&mut self, t: Seconds) {
        for route in &mut self.routes {
            if let ModulationSource::Envelope(ref mut env) = route.source {
                env.gate_off(t);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvelopeStage {
    pub kind: StageKind,
    pub start_level: Float,
    pub end_level: Float,
}

#[derive(Debug, Clone)]
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
        if self.gate_on_time == None {
            self.gate_on_time = Some(time);
            self.gate_off_time = None;
        }
    }

    pub fn gate_off(&mut self, time: Float) {
        if self.gate_off_time == None {
            self.gate_off_time = Some(time);
        }
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
