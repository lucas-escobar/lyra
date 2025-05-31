use lyra::compose::{
    Attributes, AttributesOptions, GeneralMidiInstrument,
    MusescoreInstrumentSound, Score, ScoreOptions,
};

use std::fs::{create_dir_all, File};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut score = Score::new(ScoreOptions {
        title: "Minimal Compose",
        composer: "Lucas Escobar",
        arranger: "Lucas Escobar",
        source: None,
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

        let attr = Attributes::new(AttributesOptions::default());

        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 150);
            m.add_dynamics("mf");
            m.add_note("C4:h~");
            m.add_note("C4:q");
            m.add_rest("q");
        });

        p.add_measure(None, |m| {
            m.add_note("C#2:w~");
        });

        p.add_measure(None, |m| {
            m.add_note("C#2:w");
        });
    })?;

    score.add_part("Kick Drum", |p| {
        p.add_instrument(
            "Kick",
            Some(
                GeneralMidiInstrument::ElectricBassPicked
                    .program_change_number(),
            ),
            Some(MusescoreInstrumentSound::DrumBassdrum.to_id()),
        );
        let attr = Attributes::new(AttributesOptions {
            clefs: vec!["percussion"],
            ..AttributesOptions::default()
        });

        p.add_measure(Some(attr), |m| {
            m.add_metronome("quarter", 150);
            m.add_dynamics("mf");
            // For one line clef (percussion) the note on the line
            // is E4
            m.add_note("E4:q");
            m.add_note("E4:q");
            m.add_note("E4:q");
            m.add_note("E4:q");
        });
    })?;

    let output_path = Path::new("output/min_compose.xml");
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)?;
    }
    let file = File::create(&output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    score.write_to(&mut writer)?;
    score.write_to(&mut std::io::stdout())?;
    Ok(())
}
