use lyra::compose::{Attributes, AttributesOptions, Score, ScoreOptions};
use lyra::render::{
    save_to_wav, Distortion, Instrument, KickDrum, ParametricDecayEnvelope,
    RenderContext,
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

    let kick = KickDrum {
        amp_env: ParametricDecayEnvelope {
            start: 1.0,
            end: 0.0,
            exponent: 3.0, // Fast exponential decay
        },
        freq_env: ParametricDecayEnvelope {
            start: 150.0,  // Starts high for transient punch
            end: 50.0,     // Drops to bass
            exponent: 4.0, // Sharp downward pitch drop
        },
        distortion: Some(Distortion { drive: 2.5 }), // Adds grit
        transient: None, // Optional: You can later add a snappy click here
    };

    let context = RenderContext { sample_rate: 44100 };
    let buffer = kick.render_part(&score.parts[0], &context);

    let output_path = Path::new("output/drum.wav");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }

    save_to_wav(&buffer, None, context.sample_rate as u32, output_path);

    Ok(())
}
