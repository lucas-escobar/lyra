use lyra::compose::{AttributesCreateInfo, Score, ScoreCreateInfo};
use lyra::process::{
    GainEffect, LowPassFilter, PanEffect, Processor, StereoBuffer, Track,
};
use lyra::render::{
    save_to_wav, Distortion, HiHat, KickDrum, ParametricDecayEnvelope,
    RenderContext, SnareDrum, UnpitchedInstrument,
};

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use std::fs::create_dir_all;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = Score::new(ScoreCreateInfo {
        title: "Drum Track",
        composer: "Lucas Escobar",
        arranger: "Lucas Escobar",
        source: None,
    });

    score.part("Kick", |p| {
        p.measure(|m| {
            m.attributes(AttributesCreateInfo {
                clefs: vec!["percussion"],
                ..AttributesCreateInfo::default()
            });
            m.metronome("quarter", 90);
            m.dynamics("mf");
            m.note("E4:q");
            m.note("E4:q");
            m.note("E4:q");
            m.note("E4:q");
        });
    })?;

    score.part("Snare", |p| {
        p.measure(|m| {
            m.attributes(AttributesCreateInfo {
                clefs: vec!["percussion"],
                ..AttributesCreateInfo::default()
            });
            m.metronome("quarter", 90);
            m.dynamics("mf");
            m.rest("q");
            m.note("E4:q");
            m.rest("q");
            m.note("E4:q");
        });
    })?;

    score.part("High Hat", |p| {
        p.measure(|m| {
            m.attributes(AttributesCreateInfo {
                clefs: vec!["percussion"],
                ..AttributesCreateInfo::default()
            });
            m.metronome("quarter", 90);
            m.dynamics("mp");
            m.note("E4:e");
            m.note("E4:e");
            m.note("E4:e");
            m.note("E4:e");
            m.note("E4:e");
            m.note("E4:e");
            m.note("E4:e");
            m.note("E4:e");
        });
    })?;

    let mut kick = KickDrum {
        amp_env: ParametricDecayEnvelope {
            start: 1.0,
            end: 0.0,
            exponent: 3.0, // Fast exponential decay
        },
        freq_env: ParametricDecayEnvelope {
            start: 120.0,  // Starts high for transient punch
            end: 30.0,     // Drops to bass
            exponent: 5.0, // Sharp downward pitch drop
        },
        distortion: Some(Distortion { drive: 1.2 }), // Adds grit
        transient: None, // Optional: You can later add a snappy click here
    };

    let mut snare = SnareDrum {
        rng: StdRng::seed_from_u64(42),
        amp_env: ParametricDecayEnvelope {
            start: 1.0,
            end: 0.0,
            exponent: 3.0,
        },
        noise_env: ParametricDecayEnvelope {
            start: 1.0,
            end: 0.0,
            exponent: 3.0,
        },
        tone_env: Some(ParametricDecayEnvelope {
            start: 1.0,
            end: 0.0,
            exponent: 6.0,
        }),
        freq: Some(180.0), // optional tonal body
        distortion: Some(Distortion { drive: 3.0 }),
        transient: None,
    };

    let mut hihat = HiHat {
        rng: StdRng::seed_from_u64(1234),
        amp_env: ParametricDecayEnvelope {
            start: 1.0,
            end: 0.0,
            exponent: 8.0,
        },
    };

    let context = RenderContext { sample_rate: 44100 };

    let mut processor = Processor {
        tracks: vec![
            Track {
                buffer: StereoBuffer::from_mono(&kick.render_part(
                    &score.part_by_name("Kick").unwrap(),
                    &context,
                )),
                effects: Some(vec![Box::new(PanEffect { pan: 0.15 })]),
            },
            Track {
                buffer: StereoBuffer::from_mono(&snare.render_part(
                    &score.part_by_name("Snare").unwrap(),
                    &context,
                )),
                effects: Some(vec![
                    Box::new(PanEffect { pan: 0.0 }),
                    Box::new(GainEffect { gain: 0.8 }),
                ]),
            },
            Track {
                buffer: StereoBuffer::from_mono(&hihat.render_part(
                    &score.part_by_name("High Hat").unwrap(),
                    &context,
                )),
                effects: Some(vec![Box::new(PanEffect { pan: -0.2 })]),
            },
        ],
        master_fx: vec![Box::new(GainEffect { gain: 0.8 })],
    };

    let out_buf = processor.process(context.sample_rate);

    let output_path = Path::new("output/drum.wav");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }

    save_to_wav(
        &out_buf.left,
        Some(&out_buf.right),
        context.sample_rate,
        output_path,
    );

    Ok(())
}
