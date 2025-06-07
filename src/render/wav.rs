use hound::{SampleFormat, WavSpec, WavWriter};
use std::fs::create_dir_all;
use std::path::Path;

/// Saves a mono or stereo buffer to a path relative to the caller of the executable
pub fn save_to_wav(
    path: &str,
    sample_rate: u32,
    left: &[f64],
    right: Option<&[f64]>,
) {
    let channels = if right.is_some() { 2 } else { 1 };
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };

    let out_path = Path::new(path);
    if let Some(parent) = out_path.parent() {
        create_dir_all(parent).expect("Parent path should be created");
    }
    let mut writer = WavWriter::create(out_path, spec).unwrap();

    match right {
        Some(right_buf) => {
            assert_eq!(
                left.len(),
                right_buf.len(),
                "Left and right buffers must be the same length"
            );
            for i in 0..left.len() {
                let l = left[i] as f32;
                let r = right_buf[i] as f32;
                writer.write_sample(l).unwrap();
                writer.write_sample(r).unwrap();
            }
        }
        None => {
            for &sample in left {
                let val = sample as f32;
                writer.write_sample(val).unwrap();
            }
        }
    }

    writer.finalize().unwrap();
}
