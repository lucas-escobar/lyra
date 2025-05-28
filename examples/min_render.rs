use lyra::compose::{Attributes, AttributesOptions, Score, ScoreOptions};
use lyra::render::{
    save_to_wav, Instrument, OscillatorType, RenderContext, Synth, ADSR,
};

use std::fs::create_dir_all;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = Score::new(ScoreOptions {
        title: "Minimal Render",
        composer: "Lucas Escobar",
        arranger: "Lucas Escobar",
        source: None,
    });

    score.add_part("Bass", |p| {
        let attr = Attributes::new(AttributesOptions::default());
        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 90);
            m.add_dynamics("mf");
            m.add_note("C4:q");
            m.add_note("D4:q");
            m.add_note("E4:q");
            m.add_note("F4:q");
        });

        p.add_measure(None, |m| {
            m.add_note("C4:q");
            m.add_rest("q");
            m.add_note("E4:q");
            m.add_rest("q");
        });

        p.add_measure(None, |m| {
            m.add_note("C4:w");
        });

        p.add_measure(None, |m| {
            m.add_chord("maj:C4:h");
            m.add_chord("min:C4:h");
        });

        p.add_measure(None, |m| {
            m.add_chord("maj7:C4:h");
            m.add_chord("min7:C4:h");
        });

        p.add_measure(None, |m| {
            m.add_note("F4:h~");
            m.add_note("F4:q");
            m.add_rest("q");
        });

        p.add_measure(None, |m| {
            m.add_note("C4:w~");
        });

        p.add_measure(None, |m| {
            m.add_note("C4:w");
        });
    })?;

    let synth = Synth {
        oscillator: OscillatorType::Sine,
        envelope: Box::new(ADSR {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.8,
            release: 0.2,
        }),
    };

    let context = RenderContext { sample_rate: 44100 * 2 };
    let buffer = synth.render_part(&score.parts[0], &context);

    let output_path = Path::new("output/minimal_render.wav");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }

    save_to_wav(&buffer, None, context.sample_rate as u32, output_path);

    Ok(())
}
