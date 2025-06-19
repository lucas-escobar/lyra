// TODO remove the compose dependencies by making intermediate representation
// of Part
use std::collections::HashMap;

use super::dsp::signal::{Noise, NoiseType, Oscillator, SignalSource};
use super::dsp::wave::Wave;
use super::dsp::{ModulationMatrix, ModulationSource, ModulationTarget};
use super::effect::EffectChain;
use super::processor::{AudioBuffer, RenderContext};
use super::types::{Float, Seconds};
use crate::compose::{DirectionType, MeasureItem, Part, StartStop};
use crate::render::wave::WaveShape;
use crate::render::{ModulationMode, ModulationRoute, ParametricEnvelope};

/// The sound an instrument generates depends on the composition of layers. Each
/// layer is mixed with the ones below it.
pub struct InstrumentLayer {
    pub signal: SignalSource,

    /// Layer-local modulations
    pub mods: Option<ModulationMatrix>,

    /// Layer-local effects
    pub fx: Option<EffectChain>,

    /// The base frequency used for unpitched layers.
    pub base_freq: Option<Float>,

    /// Volume of a given layer from 0.0 to 1.0
    pub volume: Float,
}

pub struct Instrument {
    pub layers: Vec<InstrumentLayer>,

    /// Global level modulations apply to all instrument layers
    pub mods: Option<ModulationMatrix>,

    /// Global level effects apply to all instrument layers
    pub fx: Option<EffectChain>,

    pub is_unpitched: bool,
}

impl Instrument {
    fn max_release_time(&self) -> Float {
        self.layers
            .iter()
            .flat_map(|l| l.mods.as_ref())
            .flat_map(|m| m.routes.iter())
            .filter_map(|r| match &r.source {
                ModulationSource::Envelope(env) => Some(env.release_time()),
                _ => None,
            })
            .fold(0.0, |a, b| a.max(b))
    }

    fn process_note_events(
        &mut self,
        ctx: &RenderContext,
        note_events: Vec<NoteEvent>,
        buf: &mut AudioBuffer,
    ) {
        let sr = ctx.sample_rate;

        for event in note_events {
            let dur = (event.end - event.start) + self.max_release_time();
            let n_samples = (dur * sr as Float).round() as usize;
            let start_sample = (event.start * sr as Float).round() as usize;

            // Gate ON for global mod envelopes
            //if let Some(mods) = &mut self.mods {
            //    mods.gate_on(event.start);
            //}

            // Each layer contributes to the note
            for layer in &mut self.layers {
                let mut layer_buf = AudioBuffer::Mono(vec![]);
                layer_buf.resize(n_samples);

                // Gate ON for local mod envelopes
                if let Some(mods) = &mut layer.mods {
                    //mods.gate_on(event.start);
                    mods.gate_on(0.0);
                }

                for i in 0..n_samples {
                    let t = i as Float / sr as Float;

                    // Gate OFF at note end
                    if t >= (event.end - event.start) {
                        if let Some(mods) = &mut layer.mods {
                            mods.gate_off(t);
                        }
                    }

                    // Get modulated pitch/amplitude
                    if let Some(f) = event.freq {
                        // Pitched instrument
                        let pitch = layer
                            .mods
                            .as_ref()
                            .map(|m| m.apply(ModulationTarget::Pitch, f, t))
                            .unwrap_or(f);
                        layer.signal.set_frequency(pitch);
                    } else {
                        // Unpitched instrument
                        if let Some(f) = layer.base_freq {
                            let pitch = layer
                                .mods
                                .as_ref()
                                .map(|m| m.apply(ModulationTarget::Pitch, f, t))
                                .unwrap_or(f);
                            layer.signal.set_frequency(pitch);
                        } else {
                            layer.signal.set_frequency(0.0);
                        }
                    }

                    let amp = layer
                        .mods
                        .as_ref()
                        .map(|m| {
                            m.apply(
                                ModulationTarget::Amplitude,
                                event.velocity,
                                t,
                            )
                        })
                        .unwrap_or(event.velocity);

                    let sample = layer.signal.sample(t) * amp * layer.volume;
                    layer_buf.set(i, sample);
                }

                // Apply layer effects
                if let Some(fx) = &mut layer.fx {
                    fx.process(&mut layer_buf, sr);
                }

                // Mix into final buffer
                buf.add_offset(&layer_buf, start_sample);
            }
        }

        // Apply global FX if present
        if let Some(global_fx) = &mut self.fx {
            global_fx.process(buf, sr);
        }
    }

    pub fn render_part(
        &mut self,
        part: &Part,
        ctx: &RenderContext,
    ) -> AudioBuffer {
        let mut state = RenderState::default();

        // MusicXML part will collect note events during parsing
        let mut note_events: Vec<NoteEvent> = vec![];

        // Flatten measures
        // TODO delegate this to a part fn
        let items: Vec<&MeasureItem> =
            part.measures.iter().flat_map(|m| m.items.iter()).collect();

        for item in &items {
            match item {
                MeasureItem::Note(note) => {
                    if !note.is_chord {
                        state.save_cursor();
                    }

                    let note_duration = state.ticks_to_secs(note.duration);

                    if let Some(pitch) = &note.pitch {
                        let mut event = NoteEvent {
                            velocity: state.velocity,
                            start: state.saved_cursor,
                            end: state.saved_cursor + note_duration,
                            freq: Some(pitch.to_frequency()),
                        };

                        match note.tie {
                            Some(StartStop::Start) => {
                                state.ongoing_ties.insert(
                                    pitch.to_semitone(),
                                    (state.saved_cursor, note_duration),
                                );

                                // do not push to event buffer, this is a tie
                                // so the event is not over yet
                                continue;
                            }
                            Some(StartStop::Stop) => {
                                if let Some((prev_start, prev_duration)) = state
                                    .ongoing_ties
                                    .remove(&pitch.to_semitone())
                                {
                                    // Change note event timing based on tie
                                    event.start = prev_start;
                                    event.end = prev_start
                                        + prev_duration
                                        + note_duration;
                                }
                            }
                            None => {}
                        }
                        note_events.push(event);
                    } else if let Some(_) = &note.unpitched {
                        note_events.push(NoteEvent {
                            velocity: state.velocity,
                            start: state.saved_cursor,
                            end: state.saved_cursor + note_duration,
                            freq: None,
                        });
                    }

                    if !note.is_chord {
                        state.cursor += note_duration;
                    }
                }

                MeasureItem::Direction(dir) => match &dir.kind {
                    DirectionType::Metronome { per_minute, .. } => {
                        state.tempo_bpm = *per_minute as Float;
                    }
                    DirectionType::Dynamics(dynamics) => {
                        state.velocity = dynamics.normalized_velocity();
                    }
                    _ => {}
                },

                MeasureItem::Forward(fwd) => {
                    state.cursor += state.ticks_to_secs(fwd.duration);
                }

                MeasureItem::Backup(bak) => {
                    state.cursor -= state.ticks_to_secs(bak.duration);
                }

                MeasureItem::Barline(_) => {}
            }
        }

        let mut buf = AudioBuffer::Mono(vec![]);
        buf.resize(
            (part.nominal_duration_seconds() * ctx.sample_rate as f64) as usize,
        );
        self.process_note_events(ctx, note_events, &mut buf);
        buf
    }
}

/// Mutable state used during the rendering process
pub struct RenderState {
    pub cursor: Seconds,
    pub saved_cursor: Seconds,
    pub tempo_bpm: Float,
    pub velocity: Float,
    pub divisions: u32,
    pub active_voices: Vec<NoteEvent>,
    // Pitch => (start_time_beats, total_duration_beats)
    pub ongoing_ties: HashMap<u8, (Float, Float)>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            cursor: 0.0,
            saved_cursor: 0.0,
            tempo_bpm: 120.0,
            velocity: 80.0 / 127.0, // mf
            divisions: 480,
            active_voices: vec![],
            ongoing_ties: HashMap::new(),
        }
    }
}

impl RenderState {
    pub fn seconds_per_beat(&self) -> Float {
        60.0 / self.tempo_bpm
    }

    /// Convert a duration in ticks to a duration in samples
    pub fn ticks_to_samples(&self, duration: u32, sample_rate: u32) -> usize {
        let dur_beats = duration as Float / self.divisions as Float;
        let dur_sec = dur_beats * self.seconds_per_beat();
        (dur_sec * sample_rate as Float).ceil() as usize
    }

    /// Convert a duration in ticks to a duration in samples
    pub fn ticks_to_secs(&self, duration: u32) -> Float {
        let dur_beats = duration as Float / self.divisions as Float;
        dur_beats * self.seconds_per_beat()
    }

    pub fn save_cursor(&mut self) {
        self.saved_cursor = self.cursor;
    }
}

pub struct NoteEvent {
    pub freq: Option<Float>,
    pub velocity: Float,
    pub start: Seconds,
    pub end: Seconds,
}

// CONCRETE INSTRUMENTS

pub fn kick_drum() -> Instrument {
    Instrument {
        is_unpitched: true,
        layers: vec![
            // Beater head
            InstrumentLayer {
                signal: SignalSource::Oscillator(Oscillator {
                    wave: Wave {
                        source: Box::new(WaveShape::Sine),
                        modifiers: None,
                    },
                    freq: 100.0,
                    phase: 0.0,
                }),
                volume: 0.5,
                base_freq: Some(80.0),
                mods: Some(ModulationMatrix {
                    routes: vec![
                        ModulationRoute {
                            source: ModulationSource::Envelope(
                                ParametricEnvelope::from_ahdsr(
                                    0.0005, 0.0, 0.08, 0.0, 0.0, 0.75,
                                ),
                            ),
                            target: ModulationTarget::Amplitude,
                            mode: ModulationMode::Scale,
                            depth: 1.0,
                        },
                        ModulationRoute {
                            source: ModulationSource::Envelope(
                                ParametricEnvelope::from_ahdsr(
                                    0.0005, 0.0, 0.08, 0.5, 0.0, 0.75,
                                ),
                            ),
                            target: ModulationTarget::Pitch,
                            mode: ModulationMode::Scale,
                            depth: 1.0,
                        },
                        ModulationRoute {
                            source: ModulationSource::Signal(
                                SignalSource::Noise(Noise::new(
                                    NoiseType::White,
                                    1337,
                                )),
                            ),
                            target: ModulationTarget::Pitch,
                            mode: ModulationMode::Scale,
                            depth: 1.0,
                        },
                    ],
                }),
                fx: None,
            },
            // Beater click
            //InstrumentLayer {
            //    signal: SignalSource::Noise(Noise::new(NoiseType::Brown,
            // 42)),    volume: 0.2,
            //    mods: Some(ModulationMatrix {
            //        routes: vec![ModulationRoute {
            //            source: ModulationSource::Envelope(
            //                ParametricEnvelope::from_ahdsr(
            //                    0.0, 0.002, 0.012, 0.0, 0.0, 0.75,
            //                ),
            //            ),
            //            target: ModulationTarget::Amplitude,
            //            mode: ModulationMode::Scale,
            //            depth: 1.0,
            //        }],
            //    }),
            //    fx: None,
            //    base_freq: None,
            //},
            // Reso head
            //InstrumentLayer {
            //    signal: SignalSource::Oscillator(Oscillator {
            //        wave: Wave {
            //            source: Box::new(WaveShape::Sine),
            //            modifiers: None,
            //        },
            //        freq: 40.0,
            //        phase: 0.0,
            //    }),
            //    base_freq: Some(40.0),
            //    volume: 0.3,
            //    mods: Some(ModulationMatrix {
            //        routes: vec![ModulationRoute {
            //            source: ModulationSource::Envelope(
            //                ParametricEnvelope::from_ahdsr(
            //                    0.01, 0.02, 0.2, 0.0, 0.0, 0.75,
            //                ),
            //            ),
            //            target: ModulationTarget::Amplitude,
            //            mode: ModulationMode::Scale,
            //            depth: 1.0,
            //        }],
            //    }),
            //    fx: None,
            //},
            // Far mic
            //InstrumentLayer {
            //    signal: SignalSource::Oscillator(Oscillator {
            //        wave: Wave {
            //            source: Box::new(WaveShape::Sine),
            //            modifiers: None,
            //        },
            //        freq: 120.0,
            //        phase: 0.0,
            //    }),
            //    base_freq: Some(120.0),
            //    volume: 0.1,
            //    mods: Some(ModulationMatrix {
            //        routes: vec![ModulationRoute {
            //            source: ModulationSource::Envelope(
            //                ParametricEnvelope::from_ahdsr(
            //                    0.01, 0.02, 0.2, 0.0, 0.0, 0.75,
            //                ),
            //            ),
            //            target: ModulationTarget::Amplitude,
            //            mode: ModulationMode::Scale,
            //            depth: 1.0,
            //        }],
            //    }),
            //    fx: None,
            //},
        ],
        mods: None,
        fx: None,
    }
}

pub fn snare_drum() -> Instrument {
    Instrument {
        is_unpitched: true,
        layers: vec![
            InstrumentLayer {
                volume: 0.75,
                signal: SignalSource::Oscillator(Oscillator {
                    wave: Wave {
                        source: Box::new(WaveShape::Sine),
                        modifiers: None,
                    },
                    freq: 180.0,
                    phase: 0.0,
                }),
                base_freq: Some(180.0),
                mods: Some(ModulationMatrix {
                    routes: vec![ModulationRoute {
                        source: ModulationSource::Envelope(
                            ParametricEnvelope::from_decay(1.0, 0.0, 0.2, 0.5),
                        ),
                        target: ModulationTarget::Amplitude,
                        mode: ModulationMode::Scale,
                        depth: 1.0,
                    }],
                }),
                fx: None,
            },
            InstrumentLayer {
                volume: 0.25,
                signal: SignalSource::Noise(Noise::new(NoiseType::White, 42)),
                base_freq: None,
                mods: Some(ModulationMatrix {
                    routes: vec![ModulationRoute {
                        source: ModulationSource::Envelope(
                            ParametricEnvelope::from_decay(1.0, 0.0, 0.2, 0.5),
                        ),
                        target: ModulationTarget::Amplitude,
                        mode: ModulationMode::Scale,
                        depth: 1.0,
                    }],
                }),
                fx: None,
            },
        ],
        mods: None,
        fx: None,
    }
}

pub fn hihat() -> Instrument {
    Instrument {
        is_unpitched: true,
        layers: vec![InstrumentLayer {
            signal: SignalSource::Noise(Noise::new(NoiseType::White, 1337)),
            base_freq: None,
            volume: 1.0,
            mods: Some(ModulationMatrix {
                routes: vec![ModulationRoute {
                    source: ModulationSource::Envelope(
                        ParametricEnvelope::from_decay(1.0, 0.0, 0.1, 0.3),
                    ),
                    target: ModulationTarget::Amplitude,
                    mode: ModulationMode::Scale,
                    depth: 1.0,
                }],
            }),
            fx: None,
        }],
        mods: None,
        fx: None,
    }
}
