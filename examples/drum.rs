use lyra::compose::{AttributesCreateInfo, Score, ScoreCreateInfo};
use lyra::render::{
    save_to_wav, AnyInstrument, AudioProcessor, AudioProcessorCreateInfo,
    GainEffect, HiHat, KickDrum, RenderContext, SnareDrum, Track,
    TrackCreateInfo,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const BPM: u32 = 90;
    const SAMPLE_RATE: u32 = 44100;
    const OUT_PATH: &str = "output/example/drum.wav";

    let mut score = Score::new(ScoreCreateInfo {
        title: "Drum Track",
        ..Default::default()
    });

    let attr = AttributesCreateInfo {
        clefs: ["percussion"].into(),
        ..Default::default()
    };

    score.part("Kick", |p| {
        p.measure(|m| {
            m.attributes(&attr);
            m.metronome("quarter", BPM);
            m.dynamics("mf");
            m.note_repeat("E4:q", 4);
        });
    })?;

    score.part("Snare", |p| {
        p.measure(|m| {
            m.attributes(&attr);
            m.metronome("quarter", BPM);
            m.dynamics("mf");
            m.rest("q");
            m.note("E4:q");
            m.rest("q");
            m.note("E4:q");
        });
    })?;

    score.part("High Hat", |p| {
        p.measure(|m| {
            m.attributes(&attr);
            m.metronome("quarter", BPM);
            m.dynamics("mp");
            m.note_repeat("E4:e", 8);
        });
    })?;

    // Define instruments
    let kick = KickDrum::default();
    let snare = SnareDrum::default();
    let hihat = HiHat::default();

    // Create and process tracks
    let ctx = &RenderContext { sample_rate: SAMPLE_RATE };
    AudioProcessor::new(AudioProcessorCreateInfo {
        ctx,
        tracks: vec![
            Track::new(TrackCreateInfo {
                part: score.get_part("Kick").unwrap(),
                instrument: &AnyInstrument::unpitched(kick),
                fx: None,
                ctx,
            }),
            Track::new(TrackCreateInfo {
                part: score.get_part("Snare").unwrap(),
                instrument: &AnyInstrument::unpitched(snare),
                fx: None,
                ctx,
            }),
            Track::new(TrackCreateInfo {
                part: score.get_part("High Hat").unwrap(),
                instrument: &AnyInstrument::unpitched(hihat),
                fx: None,
                ctx,
            }),
        ],
        master_fx: vec![Box::new(GainEffect { gain: 0.8 })],
    })
    .save_to(OUT_PATH);

    Ok(())
}
