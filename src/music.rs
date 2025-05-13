/// Music theory related concepts. Based around the MusicXML spec.
use crate::xml;

// TODO determine if i need to refer to xmlwriteable things generically
//pub trait XmlWritable {
//    fn write_to<W: std::io::Write>(
//        &self,
//        writer: &mut xml::Writer<W>,
//    ) -> std::io::Result<()>;
//}

pub struct Score {
    parts: Vec<Part>,
    work_title: String,
    composer: String,
    arranger: String,
    source: String,
}

impl Score {
    pub fn new(
        title: &str,
        composer: &str,
        arranger: &str,
        source: &str,
    ) -> Self {
        Self {
            parts: Vec::new(),
            work_title: title.to_string(),
            composer: composer.to_string(),
            arranger: arranger.to_string(),
            source: source.to_string(),
        }
    }

    pub fn finalize(&mut self) {
        let max_measures = self
            .parts
            .iter()
            .map(|p| p.measures.len())
            .max()
            .unwrap_or(0);
        for part in &mut self.parts {
            let missing = max_measures - part.measures.len();
            for _ in 0..missing {
                part.add_empty_measure();
            }
        }
    }

    // Write as MusicXML.
    pub fn write_to<W: std::io::Write>(
        &mut self,
        writer: &mut W,
    ) -> std::io::Result<()> {
        // Prepare the score to be used
        self.finalize();

        let mut w = xml::Writer::new(writer);

        w.raw(r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#)?;
        w.raw(r#"<!DOCTYPE score-partwise PUBLIC"#)?;
        w.raw(r#"    "-//Recordare//DTD MusicXML 4.0 Partwise//EN""#)?;
        w.raw(r#"    "http://www.musicxml.org/dtds/partwise.dtd">"#)?;

        w.open_tag(
            "score-partwise",
            Some(xml::Attributes::new(vec![("version", "4.0")])),
        )
        .unwrap();
        w.open_tag("work", None).unwrap();
        w.text_element("work-title", &self.work_title).unwrap();
        w.close_tag("work").unwrap();
        w.open_tag("identification", None).unwrap();
        w.text_element_with_attrs(
            "creator",
            &self.composer,
            xml::Attributes::new(vec![("type", "composer")]),
        )?;
        w.text_element_with_attrs(
            "creator",
            &self.arranger,
            xml::Attributes::new(vec![("type", "arranger")]),
        )?;
        w.text_element("source", &self.source).unwrap();
        w.close_tag("identification").unwrap();

        w.open_tag("part-list", None)?;
        for part in &self.parts {
            w.open_tag(
                "score-part",
                Some(xml::Attributes::new(vec![("id", &part.id)])),
            )
            .unwrap();
            w.text_element("part-name", &part.name)?;

            if let Some(inst) = &part.instrument {
                inst.score.write_to(&mut w)?;
                inst.midi.write_to(&mut w)?;
            }

            w.close_tag("score-part")?;
        }
        w.close_tag("part-list")?;

        for part in &self.parts {
            part.write_to(&mut w)?;
        }

        w.close_tag("score-partwise").unwrap();
        Ok(())
    }

    pub fn add_part<F>(
        &mut self,
        id: &str,
        name: &str,
        instrument: Option<CombinedInstrument>,
        build: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Part),
    {
        let mut part = Part::new(id, name, instrument);
        build(&mut part);
        self.parts.push(part);
        Ok(())
    }
}

#[derive(Clone)]
pub enum NaturalTone {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl NaturalTone {
    pub fn to_char(&self) -> char {
        match self {
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::F => 'F',
            Self::G => 'G',
            Self::A => 'A',
            Self::B => 'B',
        }
    }

    /// Semitone value relative to C
    pub fn to_semitone(&self) -> u8 {
        match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        }
    }
}

pub struct Part {
    measures: Vec<Measure>,
    id: String,
    name: String,
    instrument: Option<CombinedInstrument>,
}

impl Part {
    pub fn new(
        id: &str,
        name: &str,
        instrument: Option<CombinedInstrument>,
    ) -> Self {
        Self {
            measures: Vec::new(),
            id: id.to_string(),
            name: name.to_string(),
            instrument,
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag(
            "part",
            Some(xml::Attributes::new(vec![("id", &self.id)])),
        )?;
        for m in &self.measures {
            m.write_to(writer)?;
        }
        writer.close_tag("part")?;
        Ok(())
    }

    pub fn add_measure<F>(
        &mut self,
        attributes: Option<Attributes>,
        f: F,
    ) where
        F: FnOnce(&mut Measure),
    {
        let number = self.measures.len() + 1;
        let mut measure = Measure::new(number, attributes);
        f(&mut measure);
        self.measures.push(measure);
    }

    pub fn add_empty_measure(&mut self) {
        let attrs = self
            .get_current_attr()
            .expect("Cannot add empty measure: no previous attributes found");

        // Calculate ticks for full measure duration
        let beat_unit_ratio = 4.0 / attrs.time_beat_type as f32;
        let beat_ticks = (attrs.divisions as f32 * beat_unit_ratio) as u32;
        let total_divisions = attrs.time_beats as u32 * beat_ticks;

        let rest = Note::new(NoteOptions {
            pitch: None,
            kind: NoteType::Whole, // not used
            is_measure_rest: true,
            duration_override: Some(total_divisions),
            ..NoteOptions::default()
        });

        let number = self.measures.len() + 1;
        let mut measure = Measure::new(number, None);
        measure.add_item(MeasureItem::Note(rest));
        self.measures.push(measure);
    }

    // Returns most recent attributes
    pub fn get_current_attr(&self) -> Option<&Attributes> {
        for measure in self.measures.iter().rev() {
            if let Some(attrs) = &measure.attributes {
                return Some(attrs);
            }
        }
        None
    }
}

/// As per MusicXML spec
pub enum Mode {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Ionian,
    Locrian,
    None,
}

impl Mode {
    pub fn to_string(&self) -> String {
        match self {
            Self::Major => "major".to_string(),
            Self::Minor => "minor".to_string(),
            Self::Dorian => "dorian".to_string(),
            Self::Phrygian => "phrygian".to_string(),
            Self::Lydian => "lydian".to_string(),
            Self::Mixolydian => "mixolydian".to_string(),
            Self::Aeolian => "aeolian".to_string(),
            Self::Ionian => "ionian".to_string(),
            Self::Locrian => "locrian".to_string(),
            Self::None => "none".to_string(),
        }
    }
}

/// TODO may need refactor with enum
pub fn key_fifths_from_name(name: &str) -> i8 {
    match name {
        "C" => 0,
        "G" => 1,
        "D" => 2,
        "A" => 3,
        "E" => 4,
        "B" => 5,
        "F#" | "Fs" => 6,
        "C#" | "Cs" => 7,
        "F" => -1,
        "Bb" => -2,
        "Eb" => -3,
        "Ab" => -4,
        "Db" => -5,
        "Gb" => -6,
        "Cb" => -7,
        _ => panic!("Invalid key name"),
    }
}

pub enum Clef {
    Treble,
    Bass,
    Alto,
    Soprano,
    Tenor,
    // TODO Percussion,
}

impl Clef {
    pub fn to_sign(&self) -> String {
        match self {
            Self::Treble => "G".to_string(),
            Self::Bass => "F".to_string(),
            Self::Alto => "C".to_string(),
            Self::Soprano => "C".to_string(),
            Self::Tenor => "G".to_string(),
        }
    }

    pub fn to_line(&self) -> u8 {
        match self {
            Self::Treble => 2,
            Self::Bass => 4,
            Self::Alto => 3,
            Self::Soprano => 1,
            Self::Tenor => 4,
        }
    }
}

pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

pub struct Attributes {
    pub divisions: u32,
    pub key_fifths: i8, // 0 = C major, -1 = F major, 1 = G major
    pub key_mode: Mode,
    pub time_beats: u8,     // numerator
    pub time_beat_type: u8, // denominator
    pub clefs: Vec<Clef>,
    pub staves: Option<usize>,
}

pub struct AttributesOptions {
    pub key_name: String,
    pub key_mode: Mode,
    pub time_sig: TimeSignature,
    pub clefs: Vec<Clef>,
    pub divisions: u32,
}

impl Default for AttributesOptions {
    fn default() -> Self {
        Self {
            key_name: "C".to_string(),
            key_mode: Mode::Major,
            time_sig: TimeSignature {
                numerator: 4,
                denominator: 4,
            },
            clefs: vec![Clef::Treble],
            divisions: 480,
        }
    }
}

impl Attributes {
    pub fn new(opt: AttributesOptions) -> Self {
        let mut staves = None;
        if opt.clefs.len() > 1 {
            staves = Some(opt.clefs.len());
        }

        Self {
            divisions: opt.divisions,
            key_fifths: key_fifths_from_name(&opt.key_name),
            key_mode: opt.key_mode,
            time_beats: opt.time_sig.numerator,
            time_beat_type: opt.time_sig.denominator,
            staves,
            clefs: opt.clefs,
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag("attributes", None)?;
        writer.text_element("divisions", &self.divisions.to_string())?;

        writer.open_tag("key", None)?;
        writer.text_element("fifths", &self.key_fifths.to_string())?;
        writer.text_element("mode", &self.key_mode.to_string())?;
        writer.close_tag("key")?;

        writer.open_tag("time", None)?;
        writer.text_element("beats", &self.time_beats.to_string())?;
        writer.text_element("beat-type", &self.time_beat_type.to_string())?;
        writer.close_tag("time")?;

        for (index, clef) in self.clefs.iter().enumerate() {
            writer.open_tag(
                "clef",
                Some(xml::Attributes::new(vec![(
                    "number",
                    &(index + 1).to_string(),
                )])),
            )?;
            writer.text_element("sign", &clef.to_sign().to_string())?;
            writer.text_element("line", &clef.to_line().to_string())?;
            writer.close_tag("clef")?;
        }

        // TODO the <staves> element throws an error in MuseScore.
        // Check if this is my impl issue or musescores.
        //if let Some(staves) = &self.staves {
        //    writer.text_element("staves", &staves.to_string())?;
        //}

        writer.close_tag("attributes")?;
        Ok(())
    }
}

pub enum MeasureItem {
    Note(Note),
    Direction(Direction),
    Backup(Backup),
    Forward(Forward),
}

pub struct Backup {
    duration: u32,
    footnote: Option<String>, // does nothing
    level: Option<String>,    // does nothing
}

impl Backup {
    pub fn from_note_types(
        kinds: &[NoteType],
        divisions: u32,
    ) -> Self {
        // TODO note duration cannot currently include dots or time mods
        // as seen in the to_duration() fn
        let duration = kinds
            .iter()
            .map(|t| t.to_duration(divisions, None, None))
            .sum();
        Self {
            duration,
            footnote: None,
            level: None,
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag("backup", None)?;
        writer.text_element("duration", &self.duration.to_string())?;
        writer.close_tag("backup")?;
        Ok(())
    }
}

pub struct Forward {
    duration: u32,
    footnote: Option<String>,
    level: Option<String>,
    staff: Option<u8>,
}

pub struct Measure {
    number: usize,
    items: Vec<MeasureItem>,
    attributes: Option<Attributes>,
}

impl Measure {
    pub fn new(
        number: usize,
        attributes: Option<Attributes>,
    ) -> Self {
        Self {
            number,
            items: Vec::new(),
            attributes,
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag(
            "measure",
            Some(xml::Attributes::new(vec![(
                "number",
                &self.number.to_string(),
            )])),
        )?;

        if let Some(attributes) = &self.attributes {
            attributes.write_to(writer)?;
        }

        for item in &self.items {
            match item {
                MeasureItem::Note(note) => note.write_to(writer)?,
                MeasureItem::Direction(direction) => {
                    direction.write_to(writer)?
                }
                MeasureItem::Backup(backup) => backup.write_to(writer)?,
                MeasureItem::Forward(_) => panic!("Not implemented"),
            }
        }
        writer.close_tag("measure")?;
        Ok(())
    }

    /// The most generalized way to append to a measure. Other functions like
    /// add_metronome, add_note and so on use this fn internally.
    pub fn add_item(
        &mut self,
        item: MeasureItem,
    ) {
        self.items.push(item);
    }

    // TODO consider if the user can add a staff distinction or placement.
    // This fn is intended to prioritize convenience over customizability.
    // Add to beat_unit type safety to allow for dotted units and more clarity
    // to the fn user.
    pub fn add_metronome(
        &mut self,
        beat_unit: &str,
        per_minute: u32,
    ) {
        self.add_item(MeasureItem::Direction(Direction {
            kind: DirectionType::Metronome {
                beat_unit: beat_unit.to_string(),
                per_minute,
            },
            placement: Some("above".to_string()),
            staff: None,
        }))
    }

    pub fn add_dynamics(
        &mut self,
        dynamics: &str,
    ) {
        self.add_item(MeasureItem::Direction(Direction {
            kind: DirectionType::Dynamics(Dynamics::from_str(dynamics)),
            placement: Some("below".to_string()),
            staff: None,
        }));
    }

    pub fn add_note(
        &mut self,
        note_str: &str,
    ) {
        let parts: Vec<&str> = note_str.split(':').collect();
        assert!(
            parts.len() == 2,
            "add_note requires a pitch:duration notation"
        );
        self.add_item(MeasureItem::Note(Note::new(note_str.parse().unwrap())));
    }

    pub fn add_rest(
        &mut self,
        rest: &str,
    ) {
        let parts: Vec<&str> = rest.split(':').collect();
        assert!(
            parts.len() == 1,
            "add_note requires a pitch:duration notation"
        );
        self.add_item(MeasureItem::Note(Note::new(rest.parse().unwrap())));
    }

    pub fn add_chord(
        &mut self,
        chord: Chord,
        kind: NoteType,
        divisions: u32,
        staff: Option<u8>,
        voice: Option<u8>,
    ) {
        let notes = chord.to_notes(kind, divisions, staff, voice);
        for n in notes {
            self.items.push(MeasureItem::Note(n));
        }
    }
}

// TODO this name is tentative
pub struct CombinedInstrument {
    pub midi: MidiInstrument,
    pub score: ScoreInstrument,
}

// MusicXML representation of <midi-instrument>
#[derive(Default)]
pub struct MidiInstrument {
    // TODO https://www.w3.org/TR/xmlschema-2/#IDREF
    // id should be IDREF data type (ie. "P1-I1")
    pub id: String,

    pub channel: Option<u8>,
    pub name: Option<String>,
    pub bank: Option<u32>, // midi-16384 : 1-16,384
    pub program: Option<u8>,
    pub unpitched: Option<u32>, // midi-128 : 1-128
    pub volume: Option<f32>,    // percent
    pub pan: Option<f32>,       // rotation-degrees : -90 - 90 : left to right
    pub elevation: Option<f32>, // rotation-degrees : -90 - 90 : down to up
}

impl MidiInstrument {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag(
            "midi-instrument",
            Some(xml::Attributes::new(vec![("id", &self.id)])),
        )?;

        // TODO implement writing for all optional fields
        if let Some(p) = &self.program {
            writer.text_element("midi-program", &p.to_string())?;
        }
        writer.close_tag("midi-instrument")?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct ScoreInstrument {
    pub id: String,
    pub name: String,

    pub abbreviation: Option<String>,
    pub sound: Option<String>, // e.g., "strings.violin"
    pub solo: bool,
    pub ensemble: Option<u8>,
    pub virtual_library: Option<String>,
    pub virtual_name: Option<String>,
}

impl ScoreInstrument {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag(
            "score-instrument",
            Some(xml::Attributes::new(vec![("id", &self.id)])),
        )?;
        writer.text_element("instrument-name", &self.name)?;

        // TODO write optional fields
        if let Some(s) = &self.sound {
            writer.text_element("instrument-sound", s)?;
        }

        writer.close_tag("score-instrument")?;
        Ok(())
    }
}

pub enum DirectionType {
    Words(String),
    Metronome { beat_unit: String, per_minute: u32 },
    Dynamics(Dynamics),
    // TODO Wedge(Wedge),
    // Segno,
    // Coda,
    // Rehearsal,
    // Dashes,
    // Bracket,
    // Pedal,
    // OctaveShift,
}

// TODO currently supports only a single direction per direction block,
// I need to decide if supporting multiple directions per block
// are required.
pub struct Direction {
    pub kind: DirectionType,
    pub placement: Option<String>, // e.g., "above", "below"
    pub staff: Option<u8>,
}

impl Direction {
    //pub fn new(
    //    kind: DirectionType,
    //    placement: Option<&str>,

    //) -> Self {
    //    Self {
    //        kind,
    //        placement: placement.map(|s| s.to_string()),
    //    }
    //}

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        let attrs = self
            .placement
            .as_ref()
            .map(|place| vec![("placement", place.as_str())])
            .unwrap_or_default();

        writer.open_tag("direction", Some(xml::Attributes::new(attrs)))?;
        writer.open_tag("direction-type", None)?;

        match &self.kind {
            DirectionType::Words(text) => {
                writer.text_element("words", text)?;
            }
            DirectionType::Metronome {
                beat_unit,
                per_minute,
            } => {
                writer.open_tag("metronome", None)?;
                writer.text_element("beat-unit", beat_unit)?;
                writer.text_element("per-minute", &per_minute.to_string())?;

                writer.close_tag("metronome")?;
            }
            DirectionType::Dynamics(value) => {
                writer.open_tag("dynamics", None)?;
                writer.self_closing_tag(value.as_str(), None)?;
                writer.close_tag("dynamics")?;
                // TODO decide if a sound tag with dynamics attr is required
            }
        }

        writer.close_tag("direction-type")?;
        if let Some(staff) = self.staff {
            writer.text_element("staff", &staff.to_string())?;
        }
        writer.close_tag("direction")?;
        Ok(())
    }
}

pub enum Dynamics {
    PPP,
    PP,
    P,
    MP,
    MF,
    F,
    FF,
    FFF,
}

impl Dynamics {
    pub fn as_str(&self) -> &'static str {
        match self {
            Dynamics::PPP => "ppp",
            Dynamics::PP => "pp",
            Dynamics::P => "p",
            Dynamics::MP => "mp",
            Dynamics::MF => "mf",
            Dynamics::F => "f",
            Dynamics::FF => "ff",
            Dynamics::FFF => "fff",
        }
    }

    pub fn from_str(value: &str) -> Dynamics {
        match value {
            "ppp" => Dynamics::PPP,
            "pp" => Dynamics::PP,
            "p" => Dynamics::P,
            "mp" => Dynamics::MP,
            "mf" => Dynamics::MF,
            "f" => Dynamics::F,
            "ff" => Dynamics::FF,
            "fff" => Dynamics::FFF,
            _ => panic!("Unsupported dynamics string value"),
        }
    }

    /// To MIDI velocity
    pub fn velocity(&self) -> u8 {
        match self {
            Dynamics::PPP => 16,
            Dynamics::PP => 33,
            Dynamics::P => 49,
            Dynamics::MP => 64,
            Dynamics::MF => 80,
            Dynamics::F => 96,
            Dynamics::FF => 112,
            Dynamics::FFF => 127,
        }
    }
}

pub enum Stem {
    Up,
    Down,
    None,
    Double,
}

// TODO turn this into a struct with beam-value ranging from 1-8 for Eighth
// to 1024ths
pub enum Beam {
    Begin,
    Continue,
    End,
    ForwardHook,
    BackwardHook,
}

/// MusicXML attribute type
pub enum StartStop {
    Start,
    Stop,
}

impl StartStop {
    pub fn to_string(&self) -> String {
        match self {
            Self::Start => "start".to_string(),
            Self::Stop => "stop".to_string(),
        }
    }
}

// TODO embed in Note struct
pub struct Notations {
    items: Vec<NotationType>,
    footnote: Option<String>,
    level: Option<String>,
}

pub enum NotationType {
    Tied,
    Slur,
    Tuplet(Tuplet),
    Glissando,
    Slide,
    Ornaments,
    Technical,
    Articulations,
    // TODO Check if there is naming conflict for Dynamics,
    Fermata,
    Arpeggiate,
    NonArpeggiate,
    AccidentalMark,
    OtherNotation,
}

// TODO implement optional attributes from
// https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/tuplet/
pub struct Tuplet {
    kind: StartStop,
}

impl NotationType {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        match self {
            Self::Tuplet(t) => writer.self_closing_tag(
                "tuplet",
                Some(xml::Attributes::new(vec![("type", &t.kind.to_string())])),
            ),
            _ => panic!("Notation type not implemented"),
        }
    }
}

impl Notations {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag("notations", None)?;
        for n in &self.items {
            n.write_to(writer)?;
        }
        writer.close_tag("notations")?;
        Ok(())
    }
}

pub struct TimeModification {
    actual_note_beats: u8,
    normal_note_beats: u8,
    normal_note_type: Option<NoteType>,
}

impl TimeModification {
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag("time-modification", None)?;
        writer.text_element(
            "actual-notes",
            &self.actual_note_beats.to_string(),
        )?;
        writer.text_element(
            "normal-notes",
            &self.normal_note_beats.to_string(),
        )?;
        if let Some(t) = &self.normal_note_type {
            writer.text_element("normal-type", &t.to_string())?;
        }
        writer.close_tag("time-modification")?;
        Ok(())
    }
}

pub struct Note {
    kind: NoteType,
    pitch: Option<Pitch>,
    is_chord: bool,
    duration: u32,
    staff: Option<u8>,
    voice: Option<u8>,
    time_mod: Option<TimeModification>,
    notations: Option<Notations>,
    dots: Option<u8>,
    is_measure_rest: bool,
}

pub struct NoteOptions {
    pub kind: NoteType,
    pub divisions: u32,
    pub is_chord: bool,
    pub pitch: Option<Pitch>,
    pub staff: Option<u8>,
    pub voice: Option<u8>,
    pub time_mod: Option<TimeModification>,
    pub notations: Option<Notations>,
    pub dots: Option<u8>,
    pub is_measure_rest: bool,

    // TODO this is to handle the case of measure rests
    pub duration_override: Option<u32>,
}

impl Default for NoteOptions {
    fn default() -> Self {
        Self {
            kind: NoteType::Quarter,
            pitch: None,
            divisions: 480,
            is_chord: false,
            staff: None,
            voice: None,
            time_mod: None,
            notations: None,
            dots: None,
            is_measure_rest: false,
            duration_override: None,
        }
    }
}

impl std::str::FromStr for NoteOptions {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() > 2 {
            return Err("Expected format like C#4:h.".to_string());
        }
        let (pitch_opt, duration_str) = match parts.len() {
            1 => (None, parts[0]), // Rest: "h." format
            2 => {
                let pitch = parts[0]
                    .parse::<Pitch>()
                    .map(Some)
                    .map_err(|e| format!("Invalid pitch: {}", e))?;
                (pitch, parts[1])
            }
            _ => return Err("Expected format like C#4:h. or h.".to_string()),
        };
        //let pitch = parts[0].parse::<Pitch>()?;

        // Parse duration and dots
        let mut chars = duration_str.chars();
        let type_char = chars.next().ok_or("Missing note type")?;
        let kind = NoteType::from_char(type_char);
        let dot_count = chars.filter(|&c| c == '.').count();

        Ok(NoteOptions {
            pitch: pitch_opt,
            kind,
            dots: if dot_count > 0 {
                Some(dot_count as u8)
            } else {
                None
            },
            ..NoteOptions::default()
        })
    }
}

impl Note {
    pub fn new(opt: NoteOptions) -> Self {
        // Measure rests should ignore the normal duration calculation.
        // Measure rests will have a duration value that fills the whole
        // measure.

        let mut duration = 0;
        if opt.is_measure_rest {
            if let Some(d) = opt.duration_override {
                duration = d;
            } else {
                panic!("Duration override must be used for measure rests");
            }
        } else {
            duration = opt.kind.to_duration(
                opt.divisions,
                opt.dots,
                opt.time_mod.as_ref(),
            );
        }

        Self {
            kind: opt.kind.clone(),
            pitch: opt.pitch,
            is_chord: opt.is_chord,
            duration,
            //duration: opt.kind.to_duration(
            //    opt.divisions,
            //    opt.dots,
            //    opt.time_mod.as_ref(),
            //),
            staff: opt.staff,
            voice: opt.voice,
            time_mod: opt.time_mod,
            notations: opt.notations,
            dots: opt.dots,
            is_measure_rest: opt.is_measure_rest,
        }
    }

    pub fn is_rest(self) -> bool {
        self.pitch.is_none()
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag("note", None)?;
        if self.is_chord {
            writer.self_closing_tag("chord", None)?;
        }
        if let Some(pitch) = &self.pitch {
            pitch.write_to(writer)?;
        } else {
            let rest_attrs = if self.is_measure_rest {
                Some(xml::Attributes::new(vec![("measure", "yes")]))
            } else {
                None
            };
            writer.self_closing_tag("rest", rest_attrs)?;
        }
        writer.text_element("duration", &self.duration.to_string())?;

        if !self.is_measure_rest {
            writer.text_element("type", &self.kind.to_string())?;
        }

        if let Some(staff) = self.staff {
            writer.text_element("staff", &staff.to_string())?;
        }
        if let Some(voice) = self.voice {
            writer.text_element("voice", &voice.to_string())?;
        }
        if let Some(tm) = &self.time_mod {
            tm.write_to(writer)?;
        }
        if let Some(dots) = self.dots {
            for _ in 0..dots {
                writer.self_closing_tag("dot", None)?;
            }
        }
        if let Some(notations) = &self.notations {
            notations.write_to(writer)?;
        }
        writer.close_tag("note")?;
        Ok(())
    }
}

#[derive(Clone)]
pub enum NoteType {
    Maxima,
    Long,
    Breve,
    Whole,        // semibreve
    Half,         // minim
    Quarter,      // crotchet
    Eighth,       // quaver
    Sixteenth,    // semiquaver
    ThirtySecond, // demisemiquaver
    SixtyFourth,  // hemidemisemiquaver
    OneTwentyEighth,
    TwoFiftySixth,
    FiveTwelvth,
    TenTwentyFourth,
}

impl NoteType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Maxima => "maxima".to_string(),
            Self::Long => "long".to_string(),
            Self::Breve => "breve".to_string(),
            Self::Whole => "whole".to_string(),
            Self::Half => "half".to_string(),
            Self::Quarter => "quarter".to_string(),
            Self::Eighth => "eighth".to_string(),
            Self::Sixteenth => "16th".to_string(),
            Self::ThirtySecond => "32nd".to_string(),
            Self::SixtyFourth => "64th".to_string(),
            Self::OneTwentyEighth => "128th".to_string(),
            Self::TwoFiftySixth => "256th".to_string(),
            Self::FiveTwelvth => "512th".to_string(),
            Self::TenTwentyFourth => "1024th".to_string(),
        }
    }

    pub fn to_duration(
        &self,
        divisions: u32,
        dots: Option<u8>,
        time_mod: Option<&TimeModification>,
    ) -> u32 {
        let base = match self {
            Self::Maxima => divisions * 32,
            Self::Long => divisions * 16,
            Self::Breve => divisions * 8,
            Self::Whole => divisions * 4,
            Self::Half => divisions * 2,
            Self::Quarter => divisions,
            Self::Eighth => divisions / 2,
            Self::Sixteenth => divisions / 4,
            Self::ThirtySecond => divisions / 8,
            Self::SixtyFourth => divisions / 16,
            Self::OneTwentyEighth => divisions / 32,
            Self::TwoFiftySixth => divisions / 64,
            Self::FiveTwelvth => divisions / 128,
            Self::TenTwentyFourth => divisions / 256,
        };

        let dotted = match dots.unwrap_or(0) {
            0 => base,
            1 => base + (base / 2),
            2 => base + (base / 2) + (base / 4),
            3 => base + (base / 2) + (base / 4) + (base / 8),
            _ => panic!("Unsupported number of dots: >3"),
        };

        if let Some(tm) = time_mod {
            dotted * tm.normal_note_beats as u32 / tm.actual_note_beats as u32
        } else {
            dotted
        }
    }

    fn from_char(c: char) -> Self {
        match c {
            'w' => NoteType::Whole,
            'h' => NoteType::Half,
            'q' => NoteType::Quarter,
            'e' => NoteType::Eighth,
            's' => NoteType::Sixteenth,
            't' => NoteType::ThirtySecond,
            _ => panic!("Unexpected char found for NoteType parsing"),
        }
    }
}

#[derive(Clone)]
pub struct Pitch {
    pub step: NaturalTone,
    pub octave: i8,
    pub alter: Option<i8>,
}

impl Pitch {
    /// Semitone value relative to C-1 (midi value of 0)
    pub fn to_semitone(&self) -> u8 {
        let mut semitone = self.step.to_semitone() as i8;
        if let Some(alter) = self.alter {
            semitone += alter;
        }
        semitone += (self.octave + 1) * 12;
        if semitone < 0 || semitone > 127 {
            panic!("semitone out of midi range");
        }
        semitone as u8
    }

    /// From a semitone value relative to C-1 (MIDI value of 0)
    pub fn from_semitone(
        semitone: u8,
        prefer_flat: bool,
    ) -> Self {
        assert!(semitone <= 127, "semitone out of MIDI range");

        let octave = (semitone / 12) as i8 - 1;
        let semitone_in_octave = semitone % 12;

        if prefer_flat {
            match semitone_in_octave {
                0 => Pitch {
                    step: NaturalTone::C,
                    octave,
                    alter: None,
                },
                1 => Pitch {
                    step: NaturalTone::D,
                    octave,
                    alter: Some(-1),
                }, // D♭
                2 => Pitch {
                    step: NaturalTone::D,
                    octave,
                    alter: None,
                },
                3 => Pitch {
                    step: NaturalTone::E,
                    octave,
                    alter: Some(-1),
                }, // E♭
                4 => Pitch {
                    step: NaturalTone::E,
                    octave,
                    alter: None,
                },
                5 => Pitch {
                    step: NaturalTone::F,
                    octave,
                    alter: None,
                },
                6 => Pitch {
                    step: NaturalTone::G,
                    octave,
                    alter: Some(-1),
                }, // G♭
                7 => Pitch {
                    step: NaturalTone::G,
                    octave,
                    alter: None,
                },
                8 => Pitch {
                    step: NaturalTone::A,
                    octave,
                    alter: Some(-1),
                }, // A♭
                9 => Pitch {
                    step: NaturalTone::A,
                    octave,
                    alter: None,
                },
                10 => Pitch {
                    step: NaturalTone::B,
                    octave,
                    alter: Some(-1),
                }, // B♭
                11 => Pitch {
                    step: NaturalTone::B,
                    octave,
                    alter: None,
                },
                _ => unreachable!("invalid semitone in octave"),
            }
        } else {
            match semitone_in_octave {
                0 => Pitch {
                    step: NaturalTone::C,
                    octave,
                    alter: None,
                },
                1 => Pitch {
                    step: NaturalTone::C,
                    octave,
                    alter: Some(1),
                }, // C♯
                2 => Pitch {
                    step: NaturalTone::D,
                    octave,
                    alter: None,
                },
                3 => Pitch {
                    step: NaturalTone::D,
                    octave,
                    alter: Some(1),
                }, // D♯
                4 => Pitch {
                    step: NaturalTone::E,
                    octave,
                    alter: None,
                },
                5 => Pitch {
                    step: NaturalTone::F,
                    octave,
                    alter: None,
                },
                6 => Pitch {
                    step: NaturalTone::F,
                    octave,
                    alter: Some(1),
                }, // F♯
                7 => Pitch {
                    step: NaturalTone::G,
                    octave,
                    alter: None,
                },
                8 => Pitch {
                    step: NaturalTone::G,
                    octave,
                    alter: Some(1),
                }, // G♯
                9 => Pitch {
                    step: NaturalTone::A,
                    octave,
                    alter: None,
                },
                10 => Pitch {
                    step: NaturalTone::A,
                    octave,
                    alter: Some(1),
                }, // A♯
                11 => Pitch {
                    step: NaturalTone::B,
                    octave,
                    alter: None,
                },
                _ => unreachable!("invalid semitone in octave"),
            }
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag("pitch", None)?;
        writer.text_element("step", &self.step.to_char().to_string())?;
        if let Some(alter) = self.alter {
            writer.text_element("alter", &alter.to_string())?;
        }
        writer.text_element("octave", &self.octave.to_string())?;
        writer.close_tag("pitch")?;
        Ok(())
    }
}

// TODO currently does not support negative octaves
impl std::str::FromStr for Pitch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err("String too short".into());
        }

        let chars: Vec<char> = s.chars().collect();

        // TODO this might be a reimplementation of a naturaltone fn
        let step = match chars[0].to_ascii_uppercase() {
            'C' => NaturalTone::C,
            'D' => NaturalTone::D,
            'E' => NaturalTone::E,
            'F' => NaturalTone::F,
            'G' => NaturalTone::G,
            'A' => NaturalTone::A,
            'B' => NaturalTone::B,
            _ => return Err("Invalid step".into()),
        };

        // If the second char is alter, octave value is third char
        let (alter, octave_start) = match chars.get(1) {
            Some('#') => (Some(1), 2),
            Some('b') => (Some(-1), 2),
            _ => (None, 1),
        };

        let octave: i8 =
            s[octave_start..].parse().map_err(|_| "Invalid octave")?;

        Ok(Pitch {
            step,
            alter,
            octave,
        })
    }
}

pub struct Chord {
    root: Pitch,
    pitches: Vec<Pitch>,
    quality: ChordQuality,
    transform: Option<ChordTransform>,
}

impl Chord {
    pub fn new(
        root: Pitch,
        quality: ChordQuality,
        transform: Option<ChordTransform>,
    ) -> Self {
        let mut pitches = Vec::new();
        let rel_semitones = quality.to_semitones();
        for s in rel_semitones {
            let abs = s + root.to_semitone();
            pitches.push(Pitch::from_semitone(abs, false));
        }
        Self {
            root,
            pitches,
            quality,
            transform,
        }
    }

    // Convert chord pitches into MusicXML compatable notes
    pub fn to_notes(
        &self,
        note_kind: NoteType,
        divisions: u32,
        staff: Option<u8>,
        voice: Option<u8>,
    ) -> Vec<Note> {
        let mut notes = Vec::new();
        for (i, p) in self.pitches.iter().enumerate() {
            let is_chord = i != 0;
            notes.push(Note::new(NoteOptions {
                pitch: Some(p.clone()),
                is_chord,
                kind: note_kind.clone(),
                divisions,
                staff,
                voice,
                ..NoteOptions::default()
            }));
        }
        notes
    }
}

pub struct ChordTransform {
    inversion: u8,
    omit: Vec<u8>, // list of indices to remove (in root position)
}

pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
    Major7,
    Minor7,
    Minor7b5,
    MinorMajor7,
    Dominant7,
    Major6,
    Minor6,
    Major9,
    Minor9,
    Dominant9,
}

impl ChordQuality {
    /// Semitones are relative to some chord root. Listed in root position.
    pub fn to_semitones(&self) -> Vec<u8> {
        match self {
            Self::Major => vec![0, 4, 7],
            Self::Minor => vec![0, 3, 7],
            Self::Diminished => vec![0, 3, 6],
            Self::Augmented => vec![0, 4, 8],
            Self::Sus2 => vec![0, 2, 7],
            Self::Sus4 => vec![0, 5, 7],
            Self::Major7 => vec![0, 4, 7, 11],
            Self::Minor7 => vec![0, 3, 7, 10],
            Self::Minor7b5 => vec![0, 3, 6, 10],
            Self::MinorMajor7 => vec![0, 3, 7, 11],
            Self::Major6 => vec![0, 4, 7, 9],
            Self::Minor6 => vec![0, 3, 7, 9],
            Self::Dominant7 => vec![0, 4, 7, 10],
            Self::Major9 => vec![0, 4, 7, 11, 14],
            Self::Minor9 => vec![0, 3, 7, 10, 14],
            Self::Dominant9 => vec![0, 4, 7, 10, 14],
        }
    }
}

pub struct Scale;
pub struct Voice;
