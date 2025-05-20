use lyra::compose::{
    Attributes, AttributesOptions, GeneralMidiInstrument,
    MusescoreInstrumentSound, Score, ScoreOptions,
};

use std::fs::{create_dir_all, File};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = Score::new(ScoreOptions {
        title: "Cave",
        composer: "Ikuko Mimori",
        arranger: "Lucas Escobar",
        source: Some("From Pokemon Snap (N64)"),
    });

    score.add_part("Bass", |p| {
        p.add_instrument(
            "Bass",
            Some(
                GeneralMidiInstrument::ElectricBassPicked
                    .program_change_number(),
            ),
            Some(MusescoreInstrumentSound::PluckBass.to_id()),
        );

        let attr = Attributes::new(AttributesOptions {
            key_name: "C#",
            key_mode: "major",
            time_sig: "12/8",
            clefs: vec!["bass"],
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
    })?;

    score.add_part("Keyboard", |p| {
        p.add_instrument(
            "Keyboard",
            Some(
                GeneralMidiInstrument::AcousticGrandPiano
                    .program_change_number(),
            ),
            Some(MusescoreInstrumentSound::KeyboardPianoGrand.to_id()),
        );

        let attr = Attributes::new(AttributesOptions {
            key_name: "C#",
            key_mode: "major",
            time_sig: "12/8",
            clefs: vec!["treble"],
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
    })?;

    score.add_part("Arpeggios", |p| {
        p.add_instrument(
            "Harp",
            Some(
                GeneralMidiInstrument::AcousticGrandPiano
                    .program_change_number(),
            ),
            Some(MusescoreInstrumentSound::PluckHarp.to_id()),
        );

        let attr = Attributes::new(AttributesOptions {
            key_name: "C#",
            key_mode: "major",
            time_sig: "12/8",
            clefs: vec!["treble", "bass"],
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
    })?;

    score.add_part("Voice Harmony", |p| {
        p.add_instrument(
            "Voice",
            Some(
                GeneralMidiInstrument::AcousticGrandPiano
                    .program_change_number(),
            ),
            Some(MusescoreInstrumentSound::VoiceAlto.to_id()),
        );

        let attr = Attributes::new(AttributesOptions {
            key_name: "C#",
            key_mode: "major",
            time_sig: "12/8",
            clefs: vec!["treble"],
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
    })?;

    score.add_part("Voice Harmony 2", |p| {
        p.add_instrument(
            "Voice",
            Some(
                GeneralMidiInstrument::AcousticGrandPiano
                    .program_change_number(),
            ),
            Some(MusescoreInstrumentSound::VoiceAlto.to_id()),
        );

        let attr = Attributes::new(AttributesOptions {
            key_name: "C#",
            key_mode: "major",
            time_sig: "12/8",
            clefs: vec!["treble"],
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
    })?;

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
