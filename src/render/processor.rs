#[derive(Clone)]
/// Immutable configuration use for rendering
pub struct RenderContext {
    pub sample_rate: u32,
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
