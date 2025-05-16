use mtk::midi::Instrument;
use mtk::musescore::MusescoreInstrumentSound;
use mtk::music::{
    Attributes, AttributesOptions, Clef, CombinedInstrument, MidiInstrument,
    Mode, Score, ScoreInstrument, TimeSignature,
};

use std::fs::{create_dir_all, File};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                sound: Some(MusescoreInstrumentSound::PluckBass.to_id()),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
            let attr = Attributes::new(AttributesOptions {
                key_name: "C#".into(),
                key_mode: Mode::Major,
                time_sig: TimeSignature { numerator: 12, denominator: 8 },
                clefs: vec![Clef::Bass],
                divisions,
            });

            p.add_measure(Some(attr), |m| {
                m.add_metronome("quarter", 150);
                m.add_dynamics("mp");
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

    score.add_part(
        "P2",
        "Keyboard",
        Some(CombinedInstrument {
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
                    MusescoreInstrumentSound::KeyboardPianoGrand.to_id(),
                ),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
            let attr = Attributes::new(AttributesOptions {
                key_name: "C#".into(),
                key_mode: Mode::Major,
                time_sig: TimeSignature { numerator: 12, denominator: 8 },
                clefs: vec![Clef::Treble],
                divisions,
            });

            p.add_measure(Some(attr), |m| {
                m.add_dynamics("mf");
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
        },
    )?;

    score.add_part(
        "P3",
        "Arpeggios",
        Some(CombinedInstrument {
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
                sound: Some(MusescoreInstrumentSound::PluckHarp.to_id()),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
            let attr = Attributes::new(AttributesOptions {
                key_name: "C#".into(),
                key_mode: Mode::Major,
                time_sig: TimeSignature { numerator: 12, denominator: 8 },
                clefs: vec![Clef::Treble],
                divisions,
            });

            p.add_measure(Some(attr), |m| {
                m.add_dynamics("mp");
                m.add_note("B2:e");
                m.add_note("C#3:e");
                m.add_note("F3:e");
                m.add_note("B3:e");
                m.add_note("C#4:e");
                m.add_note("F4:e");
                m.add_note("B4:e");
                m.add_note("F4:e");
                m.add_note("C#4:e");
                m.add_note("B3:e");
                m.add_note("F3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("B2:e");
                m.add_note("C#3:e");
                m.add_note("F3:e");
                m.add_note("B3:e");
                m.add_note("C#4:e");
                m.add_note("F4:e");
                m.add_note("B4:e");
                m.add_note("F4:e");
                m.add_note("C#4:e");
                m.add_note("B3:e");
                m.add_note("F3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A#2:e");
                m.add_note("C#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("C#4:e");
                m.add_note("F#4:e");
                m.add_note("A#4:e");
                m.add_note("F#4:e");
                m.add_note("C#4:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("B2:e");
                m.add_note("D#3:e");
                m.add_note("A3:e");
                m.add_note("B3:e");
                m.add_note("D#4:e");
                m.add_note("A4:e");
                m.add_note("B4:e");
                m.add_note("A4:e");
                m.add_note("D#4:e");
                m.add_note("B3:e");
                m.add_note("A3:e");
                m.add_note("D#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("B2:e");
                m.add_note("D#3:e");
                m.add_note("G#3:e");
                m.add_note("B3:e");
                m.add_note("G#3:e");
                m.add_note("D#3:e");
                m.add_note("D#4:e");
                m.add_note("B3:e");
                m.add_note("G#3:e");
                m.add_note("B3:e");
                m.add_note("G#3:e");
                m.add_note("D#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A2:e");
                m.add_note("C3:e");
                m.add_note("F3:e");
                m.add_note("A3:e");
                m.add_note("C4:e");
                m.add_note("F4:e");
                m.add_note("A4:e");
                m.add_note("F4:e");
                m.add_note("C4:e");
                m.add_note("A3:e");
                m.add_note("F3:e");
                m.add_note("C3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A#2:e");
                m.add_note("C#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("C#4:e");
                m.add_note("F#4:e");
                m.add_note("A#4:e");
                m.add_note("F#4:e");
                m.add_note("C#4:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A#2:e");
                m.add_note("C#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
                m.add_note("C#4:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("G#2:e");
                m.add_note("C#3:e");
                m.add_note("F3:e");
                m.add_note("B3:e");
                m.add_note("C#4:e");
                m.add_note("F4:e");
                m.add_note("B4:e");
                m.add_note("F4:e");
                m.add_note("C#4:e");
                m.add_note("B3:e");
                m.add_note("F3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("B2:e");
                m.add_note("C#3:e");
                m.add_note("F3:e");
                m.add_note("B3:e");
                m.add_note("C#4:e");
                m.add_note("F4:e");
                m.add_note("B4:e");
                m.add_note("F4:e");
                m.add_note("C#4:e");
                m.add_note("B3:e");
                m.add_note("F3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A#2:e");
                m.add_note("C#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("C#4:e");
                m.add_note("F#4:e");
                m.add_note("A#4:e");
                m.add_note("F#4:e");
                m.add_note("C#4:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("B2:e");
                m.add_note("D#3:e");
                m.add_note("A3:e");
                m.add_note("B3:e");
                m.add_note("D#4:e");
                m.add_note("A4:e");
                m.add_note("B4:e");
                m.add_note("A4:e");
                m.add_note("D#4:e");
                m.add_note("B3:e");
                m.add_note("A3:e");
                m.add_note("D#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("B2:e");
                m.add_note("D#3:e");
                m.add_note("G#3:e");
                m.add_note("B3:e");
                m.add_note("G#3:e");
                m.add_note("D#3:e");
                m.add_note("D#4:e");
                m.add_note("B3:e");
                m.add_note("G#3:e");
                m.add_note("B3:e");
                m.add_note("G#3:e");
                m.add_note("D#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A2:e");
                m.add_note("C3:e");
                m.add_note("F3:e");
                m.add_note("A3:e");
                m.add_note("C4:e");
                m.add_note("F4:e");
                m.add_note("A4:e");
                m.add_note("F4:e");
                m.add_note("C4:e");
                m.add_note("A3:e");
                m.add_note("F3:e");
                m.add_note("C3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A#2:e");
                m.add_note("C#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("C#4:e");
                m.add_note("F#4:e");
                m.add_note("A#4:e");
                m.add_note("F#4:e");
                m.add_note("C#4:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
            });

            p.add_measure(None, |m| {
                m.add_note("A#2:e");
                m.add_note("C#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
                m.add_note("C#4:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("A#3:e");
                m.add_note("F#3:e");
                m.add_note("C#3:e");
            });
        },
    )?;

    score.add_part(
        "P4",
        "Voice Harmony",
        Some(CombinedInstrument {
            midi: MidiInstrument {
                id: "P4-I1".to_string(),
                program: Some(
                    Instrument::AcousticGrandPiano.program_change_number(),
                ),
                ..MidiInstrument::default()
            },
            score: ScoreInstrument {
                id: "P4-I1".to_string(),
                name: "Keyboard".to_string(),
                sound: Some(MusescoreInstrumentSound::VoiceAlto.to_id()),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
            let attr = Attributes::new(AttributesOptions {
                key_name: "C#".into(),
                key_mode: Mode::Major,
                time_sig: TimeSignature { numerator: 12, denominator: 8 },
                clefs: vec![Clef::Treble],
                divisions,
            });

            p.add_measure(Some(attr), |m| {
                m.add_dynamics("p");
                m.add_note("C#5:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("B4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("A#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("A4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("G#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("F4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("A#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("G#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("B4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("G#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("A#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("A4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("G#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("F4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("A#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("G#4:w.");
            });
        },
    )?;

    score.add_part(
        "P5",
        "Voice Harmony 2",
        Some(CombinedInstrument {
            midi: MidiInstrument {
                id: "P5-I1".to_string(),
                program: Some(
                    Instrument::AcousticGrandPiano.program_change_number(),
                ),
                ..MidiInstrument::default()
            },
            score: ScoreInstrument {
                id: "P5-I1".to_string(),
                name: "Keyboard".to_string(),
                sound: Some(MusescoreInstrumentSound::VoiceAlto.to_id()),
                ..ScoreInstrument::default()
            },
        }),
        |p| {
            let attr = Attributes::new(AttributesOptions {
                key_name: "C#".into(),
                key_mode: Mode::Major,
                time_sig: TimeSignature { numerator: 12, denominator: 8 },
                clefs: vec![Clef::Treble],
                divisions,
            });

            p.add_measure(Some(attr), |m| {
                m.add_dynamics("p");
                m.add_note("G#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("C#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("D#4:h.");
                m.add_note("F4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("F#4:h.");
                m.add_note("F4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("D#4:h.");
                m.add_note("D4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("C4:h.");
                m.add_note("B3:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("C#4:h.");
                m.add_note("F#4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("C#4:h.");
                m.add_note("C#4:h."); // timing needs to be fixed
            });

            p.add_measure(None, |m| {
                m.add_note("G#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("C#4:w.");
            });

            p.add_measure(None, |m| {
                m.add_note("D#4:h.");
                m.add_note("F4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("F#4:h.");
                m.add_note("F4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("D#4:h.");
                m.add_note("D4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("C4:h.");
                m.add_note("B3:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("C#4:h.");
                m.add_note("F#4:h.");
            });

            p.add_measure(None, |m| {
                m.add_note("E4:h.");
                m.add_note("C#4:h."); // timing needs to be fixed
            });
        },
    )?;

    let output_path = Path::new("output/pokemon_snap_cave.xml");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }
    let file = File::create(&output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    score.write_to(&mut writer)?;
    score.write_to(&mut std::io::stdout())?;
    Ok(())
}
