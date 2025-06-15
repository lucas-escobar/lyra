/// Rendering uses double precision floats
pub type Float = f64;

pub type Seconds = Float;
pub type Hz = Float;
pub type DutyCycle = Float;
pub type Skew = Float;

// [L, R] buffer
pub type StereoBuffer = Vec<(Float, Float)>;
pub type MonoBuffer = Vec<Float>;
