use lyra::compose::{Attributes, AttributesOptions, Score, ScoreOptions};
use lyra::process::{
    GainEffect, LowPassFilter, PanEffect, Processor, StereoBuffer, Track,
};
use lyra::render::{
    save_to_wav, Distortion, HiHat, Instrument, KickDrum,
    ParametricDecayEnvelope, RenderContext, SnareDrum,
};

use std::fs::create_dir_all;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = Score::new(ScoreOptions {
        title: "Drum Track",
        composer: "Lucas Escobar",
        arranger: "Lucas Escobar",
        source: None,
    });

    score.add_part("Kick", |p| {
        let attr = Attributes::new(AttributesOptions {
            clefs: vec!["percussion"],
            ..AttributesOptions::default()
        });

        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 90);
            m.add_dynamics("mf");
            m.add_note("E4:q");
            m.add_note("E4:q");
            m.add_note("E4:q");
            m.add_note("E4:q");
        });
    })?;

    score.add_part("Snare", |p| {
        let attr = Attributes::new(AttributesOptions {
            clefs: vec!["percussion"],
            ..AttributesOptions::default()
        });

        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 90);
            m.add_dynamics("mf");
            m.add_rest("q");
            m.add_note("E4:q");
            m.add_rest("q");
            m.add_note("E4:q");
        });
    })?;

    score.add_part("High Hat", |p| {
        let attr = Attributes::new(AttributesOptions {
            clefs: vec!["percussion"],
            ..AttributesOptions::default()
        });

        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 90);
            m.add_dynamics("mf");
            m.add_note("E4:e");
            m.add_note("E4:e");
            m.add_note("E4:e");
            m.add_note("E4:e");
            m.add_note("E4:e");
            m.add_note("E4:e");
            m.add_note("E4:e");
            m.add_note("E4:e");
        });
    })?;

    let kick = KickDrum {
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

    let snare = SnareDrum {
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

    let hihat = HiHat {
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
                buffer: StereoBuffer::from_mono(
                    &kick.render_part(&score.parts[0], &context),
                ),
                effects: Some(vec![
                    Box::new(GainEffect { gain: 1.0 }),
                    Box::new(PanEffect { pan: 0.15 }),
                    //Box::new(LowPassFilter::new(1000.0)),
                ]),
            },
            Track {
                buffer: StereoBuffer::from_mono(
                    &snare.render_part(&score.parts[1], &context),
                ),
                effects: Some(vec![
                    Box::new(GainEffect { gain: 1.0 }),
                    Box::new(PanEffect { pan: 0.0 }),
                    //Box::new(LowPassFilter::new(1000.0)),
                ]),
            },
            Track {
                buffer: StereoBuffer::from_mono(
                    &hihat.render_part(&score.parts[2], &context),
                ),
                effects: Some(vec![
                    Box::new(GainEffect { gain: 1.0 }),
                    Box::new(PanEffect { pan: -0.2 }),
                    //Box::new(LowPassFilter::new(1000.0)),
                ]),
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
