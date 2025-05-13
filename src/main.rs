mod midi;
mod musescore;
mod music;
mod xml;

//mod instrument {
//    pub struct Synthesizer;
//    pub struct BassDrum;
//    pub struct HighHat;
//    pub struct Snare;
//    pub struct SingingBowl;
//}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use crate::midi::Instrument;
    use music::{
        Attributes, AttributesOptions, Chord, ChordQuality, Clef,
        CombinedInstrument, Direction, DirectionType, Dynamics, MeasureItem,
        MidiInstrument, Mode, NaturalTone, Note, NoteType, Pitch, Score,
        ScoreInstrument, TimeSignature,
    };
    use std::fs::{create_dir_all, File};
    use std::path::Path;

    let mut score = Score::new(
        "Cave",
        "Ikuko Mimori",
        "Lucas Escobar",
        "From Pokemon Snap (N64)",
    );
    // TODO handle divisions better. this should be stored in the part and accessed
    // internally
    let divisions = 480;

    score.add_part(
        "P1",
        "Bass",
        Some(CombinedInstrument {
            midi: MidiInstrument {
                id: "P1-I1".to_string(),
                program: Some(
                    Instrument::AcousticGrandPiano.program_change_number(),
                ),
                ..MidiInstrument::default()
            },
            score: ScoreInstrument {
                id: "P1-I1".to_string(),
                name: "Bass".to_string(),
                sound: Some(
                    crate::musescore::MusescoreInstrumentSound::PluckBass
                        .to_id(),
                ),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
            let attr = Attributes::new(AttributesOptions {
                key_name: "C#".into(),
                key_mode: Mode::Major,
                time_sig: TimeSignature {
                    numerator: 12,
                    denominator: 8,
                },
                clefs: vec![Clef::Bass],
                divisions,
            });

            p.add_measure(Some(attr), |m| {
                m.add_metronome("quarter", 150);
                m.add_dynamics("mf");
                m.add_note("C#2:h.");
                m.add_note("C#2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("F2:h.");
                m.add_note("F2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("F#2:h.");
                m.add_note("F#2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("D#2:h.");
                m.add_note("D#2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("D2:h.");
                m.add_note("D2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("F2:h.");
                m.add_note("F2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("F#2:h.");
                m.add_note("F#2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("C#2:h.");
                m.add_note("F#2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("G#2:h.");
                m.add_note("C#2:q.");
                m.add_note("F2:q.");
            });

            p.add_measure(None, |m| {
                m.add_note("C#2:h.");
                m.add_note("C#2:q.");
                m.add_note("F2:e.");
                m.add_note("C#2:e.");
            });

            p.add_measure(None, |m| {
                m.add_note("F#2:h.");
                m.add_note("C#2:q.");
                m.add_note("F#2:q.");
            });

            p.add_measure(None, |m| {
                m.add_note("D#2:h.");
                m.add_note("F2:q.");
                m.add_note("C#2:q.");
            });

            p.add_measure(None, |m| {
                m.add_note("D#2:h.");
                m.add_note("F2:q.");
                m.add_note("F#2:q.");
            });

            p.add_measure(None, |m| {
                m.add_note("F2:h.");
                m.add_note("C#2:q.");
                m.add_note("F2:q.");
            });

            p.add_measure(None, |m| {
                m.add_note("F#2:h.");
                m.add_note("C#2:q.");
                m.add_note("F#2:q.");
            });

            p.add_measure(None, |m| {
                m.add_note("E2:q.");
                m.add_note("F#2:q.");
                m.add_note("G#2:q.");
                m.add_note("E2:q.");
            });

            // next section
            p.add_measure(None, |m| {
                m.add_note("D#2:h.");
                m.add_note("G#2:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("F2:h.");
                m.add_note("C#2:q.");
                m.add_note("D2:q.");
            });
        },
    )?;

    score.add_part("P2", "Keyboard", Some(CombinedInstrument {
            midi: MidiInstrument {
                id: "P2-I1".to_string(),
                program: Some(
                    Instrument::AcousticGrandPiano.program_change_number(),
                ),
                ..MidiInstrument::default()
            },
            score: ScoreInstrument {
                id: "P2-I1".to_string(),
                name: "Keyboard".to_string(),
                sound: Some(
                    crate::musescore::MusescoreInstrumentSound::KeyboardPianoGrand
                        .to_id(),
                ),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
        let attr = Attributes::new(AttributesOptions {
            key_name: "C#".into(),
            key_mode: Mode::Major,
            time_sig: TimeSignature {
                numerator: 12,
                denominator: 8,
            },
            clefs: vec![Clef::Treble],
            divisions,
        });

        p.add_measure(Some(attr), |m| {
            m.add_note("B5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("G#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");
        });

        p.add_measure(None, |m| {
            m.add_note("B5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("G#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");
        });

        p.add_measure(None, |m| {
            m.add_note("F#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("D#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");
        });

        p.add_measure(None, |m| {
            m.add_note("F#5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");

            m.add_note("F5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");

            m.add_note("D#5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");

            m.add_note("C#5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");
        });

p.add_measure(None, |m| {
            m.add_note("D#5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");

            m.add_note("E5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");

            m.add_note("F5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");

            m.add_note("F#5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");
        });





p.add_measure(None, |m| {
            m.add_note("F5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");

            m.add_note("F#5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");

            m.add_note("G#5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");

            m.add_note("F#5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");
        });

p.add_measure(None, |m| {
            m.add_note("F5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("G#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("A#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");
        });

p.add_measure(None, |m| {
            m.add_note("G#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");
        });


 // Could insert repeat with second ending here
p.add_measure(None, |m| {
            m.add_note("B5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("G#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");
        });

        p.add_measure(None, |m| {
            m.add_note("B5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("G#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");

            m.add_note("A#5:e");
            m.add_note("F5:e");
            m.add_note("C#5:e");
        });

        p.add_measure(None, |m| {
            m.add_note("F#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("D#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");
        });

        p.add_measure(None, |m| {
            m.add_note("F#5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");

            m.add_note("F5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");

            m.add_note("D#5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");

            m.add_note("C#5:e");
            m.add_note("B4:e");
            m.add_note("A4:e");
        });

p.add_measure(None, |m| {
            m.add_note("D#5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");

            m.add_note("E5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");

            m.add_note("F5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");

            m.add_note("F#5:e");
            m.add_note("B4:e");
            m.add_note("G#4:e");
        });

p.add_measure(None, |m| {
            m.add_note("F5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");

            m.add_note("F#5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");

            m.add_note("G#5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");

            m.add_note("F#5:e");
            m.add_note("C5:e");
            m.add_note("A4:e");
        });

p.add_measure(None, |m| {
            m.add_note("F5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("F#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("G#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");

            m.add_note("A#5:e");
            m.add_note("C#5:e");
            m.add_note("A#4:e");
        });


p.add_measure(None, |m| {
            m.add_note("A#5:e");
            m.add_note("E5:e");
            m.add_note("C#5:e");

            m.add_note("G#5:e");
            m.add_note("E5:e");
            m.add_note("C#5:e");

            m.add_note("G5:e");
            m.add_note("E5:e");
            m.add_note("C#5:e");

            m.add_note("G#5:e");
            m.add_note("E5:e");
            m.add_note("C#5:e");
        });



    })?;

score.add_part("P3", "Keyboard 2", Some(CombinedInstrument {
            midi: MidiInstrument {
                id: "P3-I1".to_string(),
                program: Some(
                    Instrument::AcousticGrandPiano.program_change_number(),
                ),
                ..MidiInstrument::default()
            },
            score: ScoreInstrument {
                id: "P3-I1".to_string(),
                name: "Keyboard".to_string(),
                sound: Some(
                    crate::musescore::MusescoreInstrumentSound::KeyboardPianoGrand
                        .to_id(),
                ),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
        let attr = Attributes::new(AttributesOptions {
            key_name: "C#".into(),
            key_mode: Mode::Major,
            time_sig: TimeSignature {
                numerator: 12,
                denominator: 8,
            },
            clefs: vec![Clef::Treble],
            divisions,
        });
        
    })?;


    let output_path = Path::new("output/music_score.xml");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }
    let file = File::create(&output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    score.write_to(&mut writer)?;
    score.write_to(&mut std::io::stdout())?;
    Ok(())
}
