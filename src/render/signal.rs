/// Tools for generating and modulating signals
use std::cell::RefCell;
use std::f64::consts::PI;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use super::types::Float;

type DutyCycle = Float;

pub enum WaveShape {
    Sine,
    Saw,
    Square,
    Triangle,
    Pulse(DutyCycle),
    Silence,
}

pub struct Oscillator {
    pub wave: WaveShape,
}

impl Oscillator {
    /// Return sample of waveform at specified frequency (f) and time (t)
    pub fn sample(&self, f: Float, t: Float) -> Float {
        let phase = 2.0 * PI * f * t;

        match self.wave {
            WaveShape::Sine => phase.sin(),
            WaveShape::Saw => 2.0 * (t * f - (t * f + 0.5).floor()),
            WaveShape::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveShape::Triangle => {
                2.0 * (2.0 * (t * f - (t * f + 0.5).floor())).abs() - 1.0
            }
            WaveShape::Pulse(duty) => {
                let phase_fraction = (f * t / (2.0 * PI)) % 1.0;
                if phase_fraction < duty {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveShape::Silence => 0.0,
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

pub trait ModulationSource {
    fn value_at(&self, t: Float) -> Float;
}

pub trait ModulationTarget {
    fn apply(&mut self, value: Float);
}

pub struct Modulation<'a> {
    pub source: &'a dyn ModulationSource,
    pub target: &'a mut dyn ModulationTarget,
    pub depth: Float, // -1.0..1.0
}

pub struct ModMatrix<'a> {
    pub routes: Vec<Modulation<'a>>,
}

impl<'a> ModMatrix<'a> {
    pub fn apply(&mut self, t: Float) {
        for r in &mut self.routes {
            let val = r.source.sample(t) * r.depth;
            r.target.apply_modulation(val);
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
    pub loop_point: Option<usize>, // optional: loop stage N to end until note off
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
