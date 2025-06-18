use super::instrument::Instrument;
use super::types::{Float, MonoBuffer, StereoBuffer};
use super::wav::save_to_wav;
use crate::render::EffectChain;

#[derive(Clone)]
/// Immutable configuration use for rendering
pub struct RenderContext {
    pub sample_rate: u32,
}

#[derive(Clone)]
pub enum AudioBuffer {
    Mono(MonoBuffer),
    Stereo(StereoBuffer),
}

impl AudioBuffer {
    pub fn set(&mut self, index: usize, value: Float) {
        match self {
            AudioBuffer::Mono(buf) => {
                if index < buf.len() {
                    buf[index] = value;
                }
            }
            AudioBuffer::Stereo(buf) => {
                if index < buf.len() {
                    buf[index].0 = value;
                    buf[index].1 = value;
                }
            }
        }
    }

    pub fn set_stereo(&mut self, index: usize, left: Float, right: Float) {
        if let AudioBuffer::Stereo(buf) = self {
            if index < buf.len() {
                buf[index] = (left, right);
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            AudioBuffer::Mono(v) => v.len(),
            AudioBuffer::Stereo(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn trim_end(&mut self, threshold: Float) {
        fn is_silent(sample: Float, threshold: Float) -> bool {
            sample.abs() <= threshold
        }

        match self {
            AudioBuffer::Mono(buf) => {
                while let Some(&last) = buf.last() {
                    if is_silent(last, threshold) {
                        buf.pop();
                    } else {
                        break;
                    }
                }
            }
            AudioBuffer::Stereo(buf) => {
                while let Some(&(l, r)) = buf.last() {
                    if is_silent(l, threshold) && is_silent(r, threshold) {
                        buf.pop();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    pub fn to_stereo(&self) -> AudioBuffer {
        match self {
            AudioBuffer::Mono(mono) => {
                let stereo: Vec<(Float, Float)> =
                    mono.iter().map(|&s| (s, s)).collect();
                AudioBuffer::Stereo(stereo)
            }
            AudioBuffer::Stereo(s) => AudioBuffer::Stereo(s.clone()),
        }
    }

    pub fn to_mono(&self) -> AudioBuffer {
        match self {
            AudioBuffer::Mono(mono) => AudioBuffer::Mono(mono.clone()),
            AudioBuffer::Stereo(stereo) => {
                let mono: Vec<Float> =
                    stereo.iter().map(|(l, r)| 0.5 * (l + r)).collect();
                AudioBuffer::Mono(mono)
            }
        }
    }

    pub fn max_amplitude(&self) -> Float {
        match self {
            AudioBuffer::Mono(buf) => {
                buf.iter().copied().map(Float::abs).fold(0.0, Float::max)
            }
            AudioBuffer::Stereo(buf) => buf
                .iter()
                .map(|(l, r)| Float::max(l.abs(), r.abs()))
                .fold(0.0, Float::max),
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        match self {
            AudioBuffer::Mono(v) => v.resize(new_len, 0.0),
            AudioBuffer::Stereo(v) => v.resize(new_len, (0.0, 0.0)),
        }
    }

    pub fn scale(&mut self, scale: Float) {
        match self {
            AudioBuffer::Mono(buf) => {
                for s in buf.iter_mut() {
                    *s *= scale;
                }
            }
            AudioBuffer::Stereo(buf) => {
                for (l, r) in buf.iter_mut() {
                    *l *= scale;
                    *r *= scale;
                }
            }
        }
    }

    pub fn add(&mut self, other: &AudioBuffer) {
        match (self, other) {
            (AudioBuffer::Mono(a), AudioBuffer::Mono(b)) => {
                if a.len() < b.len() {
                    a.resize(b.len(), 0.0);
                }
                for (ai, &bi) in a.iter_mut().zip(b.iter()) {
                    *ai += bi;
                }
            }

            (AudioBuffer::Stereo(a), AudioBuffer::Stereo(b)) => {
                if a.len() < b.len() {
                    a.resize(b.len(), (0.0, 0.0));
                }
                for (ai, &(bl, br)) in a.iter_mut().zip(b.iter()) {
                    ai.0 += bl;
                    ai.1 += br;
                }
            }

            (AudioBuffer::Mono(a), AudioBuffer::Stereo(b)) => {
                if a.len() < b.len() {
                    a.resize(b.len(), 0.0);
                }
                for (ai, &(bl, br)) in a.iter_mut().zip(b.iter()) {
                    *ai += 0.5 * (bl + br);
                }
            }

            (AudioBuffer::Stereo(a), AudioBuffer::Mono(b)) => {
                if a.len() < b.len() {
                    a.resize(b.len(), (0.0, 0.0));
                }
                for (ai, &bm) in a.iter_mut().zip(b.iter()) {
                    ai.0 += bm;
                    ai.1 += bm;
                }
            }
        }
    }

    pub fn add_offset(&mut self, other: &AudioBuffer, offset: usize) {
        match (self, other) {
            (AudioBuffer::Mono(a), AudioBuffer::Mono(b)) => {
                if a.len() < offset + b.len() {
                    a.resize(offset + b.len(), 0.0);
                }
                for (i, &val) in b.iter().enumerate() {
                    a[offset + i] += val;
                }
            }
            (AudioBuffer::Stereo(a), AudioBuffer::Stereo(b)) => {
                if a.len() < offset + b.len() {
                    a.resize(offset + b.len(), (0.0, 0.0));
                }
                for (i, &(l, r)) in b.iter().enumerate() {
                    a[offset + i].0 += l;
                    a[offset + i].1 += r;
                }
            }
            _ => panic!("Mismatched buffer types"),
        }
    }
}

// Track
pub struct Track {
    pub name: String,
    pub buffer: AudioBuffer,
    pub effects: Option<EffectChain>,
}

// Create track using part-instrument-fx
pub struct TrackCreateInfo<'a> {
    pub name: &'a str,
    pub part: &'a crate::compose::Part,
    pub instrument: &'a mut Instrument,
    pub fx: Option<EffectChain>,
    pub ctx: &'a RenderContext,
}

impl Track {
    pub fn new(ci: TrackCreateInfo<'_>) -> Self {
        Self {
            name: ci.name.to_string(),
            buffer: ci.instrument.render_part(ci.part, ci.ctx).to_stereo(),
            effects: ci.fx,
        }
    }

    pub fn process(&mut self, sample_rate: u32) {
        if let Some(fx) = &mut self.effects {
            fx.process(&mut self.buffer, sample_rate);
        }
    }
}

// AudioProcessor
pub struct AudioProcessor {
    pub ctx: RenderContext,
    pub tracks: Vec<Track>,
    pub master_fx: EffectChain,
}

pub struct AudioProcessorCreateInfo<'a> {
    pub ctx: &'a RenderContext,
    pub tracks: Vec<Track>,
    pub master_fx: EffectChain,
}

impl AudioProcessor {
    pub fn new(ci: AudioProcessorCreateInfo) -> Self {
        Self { ctx: ci.ctx.clone(), tracks: ci.tracks, master_fx: ci.master_fx }
    }

    pub fn save_to(&mut self, path: &str) {
        save_to_wav(path, self.ctx.sample_rate, &self.process());
    }

    pub fn process(&mut self) -> AudioBuffer {
        let mut mix = AudioBuffer::Stereo(vec![]);

        for track in &mut self.tracks {
            let mut max = track.buffer.max_amplitude();
            if max < -1.0 || max > 1.0 {
                println!(
                    "Warning: Track '{}' clips before local fx with max amp: {}",
                    track.name, max
                );
            }

            // apply effects
            track.process(self.ctx.sample_rate);

            max = track.buffer.max_amplitude();
            if max < -1.0 || max > 1.0 {
                println!(
                    "Warning: Track '{}' clips after local fx with max amp: {}",
                    track.name, max
                );
            }

            mix.add(&track.buffer);
        }

        let mut max = mix.max_amplitude();
        if max < -1.0 || max > 1.0 {
            println!(
                "Warning: Master track clips before master fx with max amp: {}",
                max
            );
        }

        for fx in &mut self.master_fx.effects {
            fx.process(&mut mix, self.ctx.sample_rate);
        }

        max = mix.max_amplitude();
        if max < -1.0 || max > 1.0 {
            println!(
                "Warning: Master track clips after master fx with max amp: {}",
                max
            );
        }

        // TODO maybe make this optional
        //mix.trim_end(0.0);

        mix
    }
}
