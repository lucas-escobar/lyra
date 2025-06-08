use std::cell::RefCell;
use std::f64::consts::PI;

use crate::render::RenderContext;

pub trait AudioEffect {
    fn process(
        &mut self,
        left: &mut [f64],
        right: &mut [f64],
        sample_rate: u32,
    );
}

// Stereo buffer for a track
pub struct StereoBuffer {
    pub left: Vec<f64>,
    pub right: Vec<f64>,
}

impl StereoBuffer {
    pub fn new(size: usize) -> Self {
        Self { left: vec![0.0; size], right: vec![0.0; size] }
    }

    pub fn from_mono(mono: &[f64]) -> Self {
        let mut stereo = StereoBuffer::new(mono.len());
        stereo.left.copy_from_slice(mono);
        stereo.right.copy_from_slice(mono);
        stereo
    }

    pub fn len(&self) -> usize {
        self.left.len()
    }

    pub fn add(&mut self, other: &StereoBuffer) {
        let len = other.len();
        if self.len() < len {
            self.left.resize(len, 0.0);
            self.right.resize(len, 0.0);
        }

        for i in 0..len {
            self.left[i] += other.left[i];
            self.right[i] += other.right[i];
        }
    }
}

// Track
pub struct Track {
    pub buffer: StereoBuffer,
    pub effects: Option<Vec<Box<dyn AudioEffect>>>,
}

// Create track using part-instrument-fx
pub struct TrackCreateInfo<'a> {
    pub part: &'a crate::compose::Part,
    pub instrument: &'a dyn crate::render::Instrument,
    pub fx: Option<Vec<Box<dyn AudioEffect>>>,
    pub ctx: &'a RenderContext,
}

impl Track {
    pub fn new(ci: TrackCreateInfo<'_>) -> Self {
        Self {
            buffer: StereoBuffer::from_mono(
                &ci.instrument.render_part(ci.part, ci.ctx),
            ),
            effects: ci.fx,
        }
    }

    pub fn process(&mut self, sample_rate: u32) {
        if let Some(effects) = &mut self.effects {
            for e in effects {
                e.process(
                    &mut self.buffer.left,
                    &mut self.buffer.right,
                    sample_rate,
                );
            }
        }
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

// AudioProcessor
pub struct AudioProcessor {
    pub ctx: RenderContext,
    pub tracks: Vec<Track>,
    pub master_fx: Vec<Box<dyn AudioEffect>>,
}

pub struct AudioProcessorCreateInfo<'a> {
    pub ctx: &'a RenderContext,
    pub tracks: Vec<Track>,
    pub master_fx: Vec<Box<dyn AudioEffect>>,
}

impl AudioProcessor {
    pub fn new(ci: AudioProcessorCreateInfo) -> Self {
        Self { ctx: ci.ctx.clone(), tracks: ci.tracks, master_fx: ci.master_fx }
    }

    pub fn process(&mut self) -> StereoBuffer {
        // TODO always uses first track as buff len
        let mut mix = StereoBuffer::new(self.tracks[0].buffer.len());

        for track in &mut self.tracks {
            track.process(self.ctx.sample_rate);
            mix.add(&track.buffer);
        }

        for fx in &mut self.master_fx {
            fx.process(&mut mix.left, &mut mix.right, self.ctx.sample_rate);
        }

        mix
    }
}
