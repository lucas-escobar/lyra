use lyra::compose::{
    AttributesCreateInfo, MusescoreInstrumentSound,
    MusicXmlInstrumentCreateInfo, Score, ScoreCreateInfo,
};
use lyra::process::{
    GainEffect, LowPassFilter, PanEffect, Processor, StereoBuffer, Track,
};
use lyra::render::{save_to_wav, OscillatorType, RenderContext, Synth, ADSR};

use std::fs::create_dir_all;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define score
    let mut score = Score::new(ScoreCreateInfo {
        title: "Oath to Order",
        composer: "Koji Kondo",
        arranger: "Lucas Escobar",
        source: Some("From The Legend of Zelda: Majora's Mask"),
    });

    score.part("Bass", |p| {
        p.instrument(MusicXmlInstrumentCreateInfo {
            part_id: p.id.clone(),
            instrument_id: 1,
            name: "Bass".into(),
            midi_program: None,
            sound: Some(MusescoreInstrumentSound::PluckBass.to_id()),
        });

        p.measure(|m| {
            m.attributes(AttributesCreateInfo {
                clefs: vec!["bass"],
                ..AttributesCreateInfo::default()
            });
            m.metronome("quarter", 60);
            m.dynamics("mf");
            m.note("D2:w");
        });

        p.measure(|m| {
            m.note("Eb2:w");
        });

        p.measure(|m| {
            m.note("D2:w");
        });

        p.measure(|m| {
            m.note("Eb2:w");
        });

        p.measure(|m| {
            m.note("E2:w");
        });

        p.measure(|m| {
            m.note("A2:h.");
            m.note("C#3:q");
        });

        p.measure(|m| {
            m.note("D3:w");
        });

        p.measure(|m| {
            m.note("C3:w");
        });

        p.measure(|m| {
            m.note("B2:w");
        });

        p.measure(|m| {
            m.note("A#2:w");
        });

        p.measure(|m| {
            m.note("A2:w");
        });

        p.measure(|m| {
            m.note("A2:w");
        });
    })?;

    score.part("Piano", |p| {
        p.instrument(MusicXmlInstrumentCreateInfo {
            part_id: p.id.clone(),
            instrument_id: 1,
            name: "Piano".into(),
            midi_program: None,
            sound: Some(MusescoreInstrumentSound::KeyboardPianoGrand.to_id()),
        });

        p.measure(|m| {
            m.attributes(AttributesCreateInfo {
                clefs: vec!["treble"],
                ..AttributesCreateInfo::default()
            });
            m.metronome("quarter", 60);
            m.dynamics("mf");
            m.rest("e");
            m.note("D3:e");
            m.note("A3:e");
            m.note("D4:e");
            m.note("F4:e");
            m.note("A4:e");
            m.note("D5:e");
            m.note("F5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("Eb3:e");
            m.note("Bb3:e");
            m.note("D4:e");
            m.note("G4:e");
            m.note("Bb4:e");
            m.note("D5:e");
            m.note("G5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("D3:e");
            m.note("A3:e");
            m.note("D4:e");
            m.note("F4:e");
            m.note("A4:e");
            m.note("D5:e");
            m.note("F5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("Eb3:e");
            m.note("Bb3:e");
            m.note("D4:e");
            m.note("G4:e");
            m.note("Bb4:e");
            m.note("D5:e");
            m.note("G5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("E3:e");
            m.note("Bb3:e");
            m.note("D4:e");
            m.note("G4:e");
            m.note("Bb4:e");
            m.note("D5:e");
            m.note("G5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("A3:e");
            m.note("Db4:e");
            m.note("E4:e");
            m.note("G4:e");
            m.note("Db5:e");
            m.note("E5:e");
            m.note("G5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("D3:e");
            m.note("A3:e");
            m.note("D4:e");
            m.note("F4:e");
            m.note("A4:e");
            m.note("D5:e");
            m.note("F5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("D3:e");
            m.note("A3:e");
            m.note("D4:e");
            m.note("F4:e");
            m.note("A4:e");
            m.note("D5:e");
            m.note("F5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("D3:e");
            m.note("B3:e");
            m.note("D4:e");
            m.note("G4:e");
            m.note("B4:e");
            m.note("D5:e");
            m.note("G5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("D3:e");
            m.note("A3:e");
            m.note("D4:e");
            m.note("F4:e");
            m.note("A4:e");
            m.note("D5:e");
            m.note("F5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("D3:e");
            m.note("A3:e");
            m.note("D4:e");
            m.note("F4:e");
            m.note("A4:e");
            m.note("D5:e");
            m.note("F5:e");
        });

        p.measure(|m| {
            m.rest("e");
            m.note("E3:e");
            m.note("G3:e");
            m.note("Db4:e");
            m.note("E4:e");
            m.note("G4:e");
            m.note("Db5:e");
            m.note("G5:e");
        });
    })?;

    // Define instruments
    let synth_saw = Synth {
        oscillator: OscillatorType::Saw,
        envelope: ADSR { attack: 0.1, decay: 0.2, sustain: 0.8, release: 0.2 },
    };

    let synth_sine = Synth {
        oscillator: OscillatorType::Sine,
        envelope: ADSR { attack: 0.1, decay: 0.2, sustain: 0.8, release: 0.2 },
    };

    // Process tracks
    let context = RenderContext { sample_rate: 44100 };
    let mut processor = Processor {
        tracks: vec![
            Track {
                buffer: StereoBuffer::from_mono(
                    &synth_saw.render_part(&score.parts[0], &context),
                ),
                effects: Some(vec![
                    Box::new(GainEffect { gain: 1.0 }),
                    Box::new(PanEffect { pan: 0.8 }),
                    Box::new(LowPassFilter::new(550.0)),
                ]),
            },
            Track {
                buffer: StereoBuffer::from_mono(
                    &synth_sine.render_part(&score.parts[1], &context),
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

    Ok(())
}
