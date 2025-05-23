use hound::{SampleFormat, WavSpec, WavWriter};
use std::path::Path;

pub fn save_to_wav(buffer: &[f64], sample_rate: u32, path: &Path) {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec).unwrap();
    for sample in buffer {
        let val = (sample * i16::MAX as f64) as i16;
        writer.write_sample(val).unwrap();
    }
    writer.finalize().unwrap();
}
