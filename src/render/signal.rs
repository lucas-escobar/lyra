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
