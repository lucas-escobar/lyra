pub struct Distortion {
    pub drive: Float,
}

impl Distortion {
    pub fn apply(&self, sample: Float) -> Float {
        // Soft clipping
        (sample * self.drive).tanh()
    }
}
