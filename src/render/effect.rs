use super::types::Float;

pub trait AudioEffect {
    fn process(
        &mut self,
        left: &mut [f64],
        right: &mut [f64],
        sample_rate: u32,
    );
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

// Gain Effect
pub struct GainEffect {
    pub gain: f64,
}

impl AudioEffect for GainEffect {
    fn process(
        &mut self,
        left: &mut [f64],
        right: &mut [f64],
        _sample_rate: u32,
    ) {
        for i in 0..left.len() {
            left[i] *= self.gain;
            right[i] *= self.gain;
        }
    }
}

// Pan Effect: -1.0 (Left), 0.0 (Center), 1.0 (Right)
pub struct PanEffect {
    pub pan: f64,
}

impl AudioEffect for PanEffect {
    fn process(
        &mut self,
        left: &mut [f64],
        right: &mut [f64],
        _sample_rate: u32,
    ) {
        let pan = self.pan.clamp(-1.0, 1.0);
        let left_gain = (1.0 - pan).sqrt() * 0.707;
        let right_gain = (1.0 + pan).sqrt() * 0.707;
        for i in 0..left.len() {
            left[i] *= left_gain;
            right[i] *= right_gain;
        }
    }
}

// One-pole Low-pass Filter
pub struct LowPassFilter {
    pub cutoff: f64,
    prev_l: f64,
    prev_r: f64,
}

impl LowPassFilter {
    pub fn new(cutoff: f64) -> Self {
        Self { cutoff, prev_l: 0.0, prev_r: 0.0 }
    }
}

impl AudioEffect for LowPassFilter {
    fn process(
        &mut self,
        left: &mut [f64],
        right: &mut [f64],
        sample_rate: u32,
    ) {
        let rc = 1.0 / (2.0 * PI * self.cutoff);
        let dt = 1.0 / sample_rate as f64;
        let alpha = dt / (rc + dt);

        for i in 0..left.len() {
            self.prev_l += alpha * (left[i] - self.prev_l);
            self.prev_r += alpha * (right[i] - self.prev_r);
            left[i] = self.prev_l;
            right[i] = self.prev_r;
        }
    }
}

// Simple saturation using tanh
pub struct SaturationEffect {
    pub drive: f64,
}

impl AudioEffect for SaturationEffect {
    fn process(
        &mut self,
        left: &mut [f64],
        right: &mut [f64],
        _sample_rate: u32,
    ) {
        for i in 0..left.len() {
            left[i] = (self.drive * left[i]).tanh();
            right[i] = (self.drive * right[i]).tanh();
        }
    }
}
