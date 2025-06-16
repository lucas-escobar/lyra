use std::fs::create_dir_all;
use std::path::Path;

use hound::{SampleFormat, WavSpec, WavWriter};

use super::processor::AudioBuffer;

pub fn save_to_wav(path: &str, sample_rate: u32, buffer: &AudioBuffer) {
    let (channels, _len) = match buffer {
        AudioBuffer::Mono(samples) => (1, samples.len()),
        AudioBuffer::Stereo(samples) => (2, samples.len()),
    };

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

    let mut writer =
        WavWriter::create(out_path, spec).expect("Failed to create WAV writer");

    match buffer {
        AudioBuffer::Mono(samples) => {
            for &s in samples {
                writer.write_sample(s as f32).unwrap();
            }
        }
        AudioBuffer::Stereo(samples) => {
            for &(l, r) in samples {
                writer.write_sample(l as f32).unwrap();
                writer.write_sample(r as f32).unwrap();
            }
        }
    }

    writer.finalize().unwrap();
}
