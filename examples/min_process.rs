use lyra::compose::{Attributes, AttributesOptions, Score, ScoreOptions};
use lyra::process::{
    GainEffect, LowPassFilter, PanEffect, Processor, StereoBuffer, Track,
};
use lyra::render::{
    save_to_wav, Instrument, OscillatorType, RenderContext, Synth, ADSR,
};

use std::fs::create_dir_all;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = Score::new(ScoreOptions {
        title: "Minimal Process",
        composer: "Lucas Escobar",
        arranger: "Lucas Escobar",
        source: None,
    });

    score.add_part("Bass", |p| {
        let attr = Attributes::new(AttributesOptions::default());
        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 90);
            m.add_dynamics("mp");
            m.add_note("C2:h");
            m.add_note("D2:h");
        });
    })?;

    score.add_part("Melody", |p| {
        let attr = Attributes::new(AttributesOptions::default());
        p.add_measure(Some(attr), |m| {
            // TODO does each part need a metronome for rendering?
            m.add_metronome("quarter", 90);
            m.add_dynamics("mf");
            m.add_note("C4:q");
            m.add_note("D4:q");
            m.add_note("E4:q");
            m.add_note("F4:q");
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
    let bass_buffer = synth.render_part(&score.parts[0], &context);
    let melody_buffer = synth.render_part(&score.parts[1], &context);

    let bass_track = Track {
        buffer: StereoBuffer::from_mono(&bass_buffer),
        effects: Some(vec![
            Box::new(GainEffect { gain: 1.0 }),
            Box::new(PanEffect { pan: -0.5 }),
            Box::new(LowPassFilter::new(1000.0)),
        ]),
    };

    let melody_track = Track {
        buffer: StereoBuffer::from_mono(&melody_buffer),
        effects: Some(vec![
            Box::new(GainEffect { gain: 1.0 }),
            Box::new(PanEffect { pan: 0.5 }),
            Box::new(LowPassFilter::new(1000.0)),
        ]),
    };

    let mut processor = Processor {
        tracks: vec![bass_track, melody_track],
        master_fx: vec![Box::new(GainEffect { gain: 0.8 })],
    };

    let out_buf = processor.process(context.sample_rate as f64);

    let output_path = Path::new("output/minimal_process.wav");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }

    save_to_wav(
        &out_buf.left,
        Some(&out_buf.right),
        context.sample_rate as u32,
        output_path,
    );

    Ok(())
}
