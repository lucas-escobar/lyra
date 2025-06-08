use std::f64::consts::PI;

use super::types::Float;

#[derive(Clone, Copy)]
pub enum OscillatorType {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl OscillatorType {
    pub fn sample(&self, freq: Float, t: Float) -> Float {
        let phase = 2.0 * PI * freq * t;
        match self {
            Self::Sine => phase.sin(),
            Self::Saw => 2.0 * (t * freq - (t * freq + 0.5).floor()),
            Self::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Triangle => {
                2.0 * (2.0 * (t * freq - (t * freq + 0.5).floor())).abs() - 1.0
            }
        }
    }
}
