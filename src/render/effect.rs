use super::types::{Float, MonoBuffer, StereoBuffer};

pub enum AudioBuffer {
    Mono(MonoBuffer),
    Stereo(StereoBuffer),
}

pub trait AudioEffect {
    fn process(&mut self, buffer: AudioBuffer, sample_rate: u32);
}

pub struct EffectChain {
    pub effects: Vec<Box<dyn AudioEffect>>,
}

impl EffectChain {
    pub fn process(&mut self, buffer: AudioBuffer, sample_rate: u32) {
        for fx in &mut self.effects {
            fx.process(buffer.clone(), sample_rate);
        }
    }
}

/// Simple gain adjustment
pub struct Gain {
    pub amount: Float,
}

impl AudioEffect for Gain {
    fn process(&mut self, buffer: AudioBuffer, _sr: u32) -> AudioBuffer {
        match buffer {
            AudioBuffer::Mono(mut buf) => {
                for s in &mut buf {
                    *s *= self.amount;
                }
                AudioBuffer::Mono(buf)
            }
            AudioBuffer::Stereo(mut buf) => {
                for (l, r) in &mut buf {
                    *l *= self.amount;
                    *r *= self.amount;
                }
                AudioBuffer::Stereo(buf)
            }
        }
    }
}

/// Basic one-pole low-pass filter
pub struct LowPass {
    pub cutoff_hz: Float,
    state: Option<(Float, Float)>, // Mono or stereo
}

impl LowPass {
    pub fn new(cutoff: Float) -> Self {
        Self { cutoff_hz: cutoff, state: None }
    }
}

impl AudioEffect for LowPass {
    fn process(
        &mut self,
        buffer: AudioBuffer,
        sample_rate: u32,
    ) -> AudioBuffer {
        let alpha = (2.0 * PI * self.cutoff_hz / sample_rate as Float).min(1.0);
        match buffer {
            AudioBuffer::Mono(mut buf) => {
                let mut last = self.state.map(|s| s.0).unwrap_or(0.0);
                for s in &mut buf {
                    last += alpha * (*s - last);
                    *s = last;
                }
                self.state = Some((last, 0.0));
                AudioBuffer::Mono(buf)
            }
            AudioBuffer::Stereo(mut buf) => {
                let (mut l_last, mut r_last) = self.state.unwrap_or((0.0, 0.0));
                for (l, r) in &mut buf {
                    l_last += alpha * (*l - l_last);
                    r_last += alpha * (*r - r_last);
                    *l = l_last;
                    *r = r_last;
                }
                self.state = Some((l_last, r_last));
                AudioBuffer::Stereo(buf)
            }
        }
    }
}

/// Simple soft saturation
pub struct Saturation;

impl AudioEffect for Saturation {
    fn process(&mut self, buffer: AudioBuffer, _sr: u32) -> AudioBuffer {
        fn saturate(x: Float) -> Float {
            (x + 0.5 * x.powi(3)).clamp(-1.0, 1.0)
        }

        match buffer {
            AudioBuffer::Mono(mut buf) => {
                for s in &mut buf {
                    *s = saturate(*s);
                }
                AudioBuffer::Mono(buf)
            }
            AudioBuffer::Stereo(mut buf) => {
                for (l, r) in &mut buf {
                    *l = saturate(*l);
                    *r = saturate(*r);
                }
                AudioBuffer::Stereo(buf)
            }
        }
    }
}

/// Hard clip distortion
pub struct Distortion {
    pub threshold: Float,
}

impl AudioEffect for Distortion {
    fn process(&mut self, buffer: AudioBuffer, _sr: u32) -> AudioBuffer {
        let th = self.threshold.abs().max(0.01);
        match buffer {
            AudioBuffer::Mono(mut buf) => {
                for s in &mut buf {
                    *s = s.clamp(-th, th);
                }
                AudioBuffer::Mono(buf)
            }
            AudioBuffer::Stereo(mut buf) => {
                for (l, r) in &mut buf {
                    *l = l.clamp(-th, th);
                    *r = r.clamp(-th, th);
                }
                AudioBuffer::Stereo(buf)
            }
        }
    }
}

/// Bitcrusher
pub struct Bitcrusher {
    pub bits: u32,
}

impl AudioEffect for Bitcrusher {
    fn process(&mut self, buffer: AudioBuffer, _sr: u32) -> AudioBuffer {
        let levels = 2u32.pow(self.bits.min(24)) as Float;
        match buffer {
            AudioBuffer::Mono(mut buf) => {
                for s in &mut buf {
                    *s = ((*s * levels).round()) / levels;
                }
                AudioBuffer::Mono(buf)
            }
            AudioBuffer::Stereo(mut buf) => {
                for (l, r) in &mut buf {
                    *l = ((*l * levels).round()) / levels;
                    *r = ((*r * levels).round()) / levels;
                }
                AudioBuffer::Stereo(buf)
            }
        }
    }
}

/// Simple delay (mono only for now)
pub struct Delay {
    pub delay_samples: usize,
    pub feedback: Float,
    pub mix: Float,
    buffer: Vec<Float>,
    pos: usize,
}

impl Delay {
    pub fn new(delay_samples: usize, feedback: Float, mix: Float) -> Self {
        Self {
            delay_samples,
            feedback,
            mix,
            buffer: vec![0.0; delay_samples],
            pos: 0,
        }
    }
}

impl AudioEffect for Delay {
    fn process(&mut self, buffer: AudioBuffer, _sr: u32) -> AudioBuffer {
        match buffer {
            AudioBuffer::Mono(mut buf) => {
                for s in &mut buf {
                    let delayed = self.buffer[self.pos];
                    self.buffer[self.pos] = *s + delayed * self.feedback;
                    *s = *s * (1.0 - self.mix) + delayed * self.mix;
                    self.pos = (self.pos + 1) % self.buffer.len();
                }
                AudioBuffer::Mono(buf)
            }
            _ => buffer, // TODO: stereo delay
        }
    }
}

/// Bypass effect for testing
pub struct Bypass;

impl AudioEffect for Bypass {
    fn process(&mut self, buffer: AudioBuffer, _sr: u32) -> AudioBuffer {
        buffer
    }
}

/// Simple Schroeder reverb with comb + allpass filters
pub struct SimpleReverb {
    comb_buffers: Vec<Vec<Float>>,
    allpass_buffers: Vec<Vec<Float>>,
    comb_indices: Vec<usize>,
    allpass_indices: Vec<usize>,
    feedback: Float,
    mix: Float,
}

impl SimpleReverb {
    pub fn new(feedback: Float, mix: Float) -> Self {
        let comb_lengths = [1116, 1188, 1277, 1356]; // Prime lengths
        let allpass_lengths = [225, 556];

        SimpleReverb {
            comb_buffers: comb_lengths.iter().map(|&l| vec![0.0; l]).collect(),
            allpass_buffers: allpass_lengths
                .iter()
                .map(|&l| vec![0.0; l])
                .collect(),
            comb_indices: vec![0; comb_lengths.len()],
            allpass_indices: vec![0; allpass_lengths.len()],
            feedback,
            mix,
        }
    }
}

impl AudioEffect for SimpleReverb {
    fn process(&mut self, buffer: AudioBuffer, _sr: u32) -> AudioBuffer {
        match buffer {
            AudioBuffer::Mono(mut buf) => {
                for s in &mut buf {
                    // === Comb Filters ===
                    let mut comb_sum = 0.0;
                    for (i, buffer) in self.comb_buffers.iter_mut().enumerate()
                    {
                        let idx = self.comb_indices[i];
                        let out = buffer[idx];
                        buffer[idx] = *s + out * self.feedback;
                        self.comb_indices[i] = (idx + 1) % buffer.len();
                        comb_sum += out;
                    }

                    // === Allpass Filters ===
                    let mut y = comb_sum;
                    for (i, buffer) in
                        self.allpass_buffers.iter_mut().enumerate()
                    {
                        let idx = self.allpass_indices[i];
                        let buf_val = buffer[idx];
                        let input = y;
                        y = -input + buf_val;
                        buffer[idx] = input + buf_val * 0.5;
                        self.allpass_indices[i] = (idx + 1) % buffer.len();
                    }

                    let wet = y;
                    *s = *s * (1.0 - self.mix) + wet * self.mix;
                }

                AudioBuffer::Mono(buf)
            }
            _ => todo!("Stereo reverb not implemented yet"),
        }
    }
}
