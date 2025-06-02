use lyra::compose::{
    Attributes, AttributesOptions, MusescoreInstrumentSound, Score,
    ScoreOptions,
};
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
        title: "Oath to Order",
        composer: "Koji Kondo",
        arranger: "Lucas Escobar",
        source: Some("From The Legend of Zelda: Majora's Mask"),
    });

    score.add_part("Bass", |p| {
        p.add_instrument(
            "Bass",
            None,
            Some(MusescoreInstrumentSound::PluckBass.to_id()),
        );

        let attr = Attributes::new(AttributesOptions {
            clefs: vec!["bass"],
            ..AttributesOptions::default()
        });

        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 60);
            m.add_dynamics("mf");
            m.add_note("D2:w");
        });

        p.add_measure(None, |m| {
            m.add_note("Eb2:w");
        });

        p.add_measure(None, |m| {
            m.add_note("D2:w");
        });

        p.add_measure(None, |m| {
            m.add_note("Eb2:w");
        });

        p.add_measure(None, |m| {
            m.add_note("E2:w");
        });

        p.add_measure(None, |m| {
            m.add_note("A2:h.");
            m.add_note("C#3:q");
        });

        p.add_measure(None, |m| {
            m.add_note("D3:w");
        });

        p.add_measure(None, |m| {
            m.add_note("C3:w");
        });

        p.add_measure(None, |m| {
            m.add_note("B2:w");
        });
    })?;

    score.add_part("Piano", |p| {
        p.add_instrument(
            "Piano",
            None,
            Some(MusescoreInstrumentSound::KeyboardPianoGrand.to_id()),
        );

        let attr = Attributes::new(AttributesOptions {
            clefs: vec!["treble"],
            ..AttributesOptions::default()
        });

        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 60);
            m.add_dynamics("mf");
            m.add_rest("e");
            m.add_note("D3:e");
            m.add_note("A3:e");
            m.add_note("D4:e");
            m.add_note("F4:e");
            m.add_note("A4:e");
            m.add_note("D5:e");
            m.add_note("F5:e");
        });

        p.add_measure(None, |m| {
            m.add_rest("e");
            m.add_note("Eb3:e");
            m.add_note("Bb3:e");
            m.add_note("D4:e");
            m.add_note("G4:e");
            m.add_note("Bb4:e");
            m.add_note("D5:e");
            m.add_note("G5:e");
        });

        p.add_measure(None, |m| {
            m.add_rest("e");
            m.add_note("D3:e");
            m.add_note("A3:e");
            m.add_note("D4:e");
            m.add_note("F4:e");
            m.add_note("A4:e");
            m.add_note("D5:e");
            m.add_note("F5:e");
        });

        p.add_measure(None, |m| {
            m.add_rest("e");
            m.add_note("Eb3:e");
            m.add_note("Bb3:e");
            m.add_note("D4:e");
            m.add_note("G4:e");
            m.add_note("Bb4:e");
            m.add_note("D5:e");
            m.add_note("G5:e");
        });
    })?;

    let synth_saw = Synth {
        oscillator: OscillatorType::Saw,
        envelope: Box::new(ADSR {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.8,
            release: 0.2,
        }),
    };

    let synth_sine = Synth {
        oscillator: OscillatorType::Sine,
        envelope: Box::new(ADSR {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.8,
            release: 0.2,
        }),
    };

    let context = RenderContext { sample_rate: 44100 };

    let bass_track = Track {
        buffer: StereoBuffer::from_mono(
            &synth_saw.render_part(&score.parts[0], &context),
        ),
        effects: Some(vec![
            Box::new(GainEffect { gain: 1.0 }),
            Box::new(PanEffect { pan: 0.8 }),
            Box::new(LowPassFilter::new(750.0)),
        ]),
    };

    let piano_track = Track {
        buffer: StereoBuffer::from_mono(
            &synth_sine.render_part(&score.parts[1], &context),
        ),
        effects: Some(vec![
            Box::new(GainEffect { gain: 1.0 }),
            Box::new(PanEffect { pan: -0.2 }),
            //Box::new(LowPassFilter::new(1000.0)),
        ]),
    };

    let mut processor = Processor {
        tracks: vec![bass_track, piano_track],
        master_fx: vec![Box::new(GainEffect { gain: 0.8 })],
    };

    let out_buf = processor.process(context.sample_rate as f64);

    let output_path = Path::new("output/oath_to_order.wav");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }

    save_to_wav(
        &out_buf.left,
        Some(&out_buf.right),
        context.sample_rate as u32,
        output_path,
    );

    save_to_wav(
        &synth_sine.render_part(&score.parts[0], &context),
        None,
        context.sample_rate as u32,
        Path::new("output/oath_to_order_bass.wav"),
    );

    Ok(())
}
