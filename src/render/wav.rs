use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::Path;

//pub fn save_to_wav(buffer: &[f64], sample_rate: u32, path: &Path) {
//    let spec = WavSpec {
//        channels: 1,
//        sample_rate,
//        bits_per_sample: 16,
//        sample_format: SampleFormat::Int,
//    };
//
//    let mut writer = WavWriter::create(path, spec).unwrap();
//    for sample in buffer {
//        let val = (sample * i16::MAX as f64) as i16;
//        writer.write_sample(val).unwrap();
//    }
//    writer.finalize().unwrap();
//}

pub fn save_to_wav(
    left: &[f64],
    right: Option<&[f64]>,
    sample_rate: u32,
    path: &Path,
) {
    let channels = if right.is_some() { 2 } else { 1 };
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec).unwrap();

    match right {
        Some(right_buf) => {
            assert_eq!(
                left.len(),
                right_buf.len(),
                "Left and right buffers must be the same length"
            );
            for i in 0..left.len() {
                let l = (left[i] * i16::MAX as f64) as i16;
                let r = (right_buf[i] * i16::MAX as f64) as i16;
                writer.write_sample(l).unwrap();
                writer.write_sample(r).unwrap();
            }
        }
        None => {
            for &sample in left {
                let val = (sample * i16::MAX as f64) as i16;
                writer.write_sample(val).unwrap();
            }
        }
    }

    writer.finalize().unwrap();
}
