use lyra::compose::{AttributesCreateInfo, Score, ScoreCreateInfo};
use lyra::render::{
    hihat, kick_drum, snare_drum, AudioProcessor, AudioProcessorCreateInfo,
    EffectChain, Gain, Pan, RenderContext, Track, TrackCreateInfo,
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

    // Create and process tracks
    let ctx = &RenderContext { sample_rate: SAMPLE_RATE };
    AudioProcessor::new(AudioProcessorCreateInfo {
        ctx,
        tracks: vec![
            Track::new(TrackCreateInfo {
                name: "Kick",
                part: score.get_part("Kick").unwrap(),
                instrument: &mut kick_drum(),
                fx: Some(EffectChain {
                    effects: vec![Box::new(Gain { amount: 1.0 })],
                }),
                ctx,
            }),
            //Track::new(TrackCreateInfo {
            //    name: "Snare",
            //    part: score.get_part("Snare").unwrap(),
            //    instrument: &mut snare_drum(),
            //    fx: Some(EffectChain {
            //        effects: vec![
            //            Box::new(Gain { amount: 0.8 }),
            //            Box::new(Pan { position: 0.15 }),
            //        ],
            //    }),
            //    ctx,
            //}),
            //Track::new(TrackCreateInfo {
            //    name: "High Hat",
            //    part: score.get_part("High Hat").unwrap(),
            //    instrument: &mut hihat(),
            //    fx: Some(EffectChain {
            //        effects: vec![
            //            Box::new(Gain { amount: 0.25 }),
            //            Box::new(Pan { position: -0.3 }),
            //        ],
            //    }),
            //    ctx,
            //}),
        ],
        master_fx: EffectChain {
            //effects: vec![Box::new(Gain { amount: 1.0 })],
            effects: vec![],
        },
    })
    .save_to(OUT_PATH);

    Ok(())
}
