/// Music theory related concepts. Based around the MusicXML spec.
use crate::compose::xml;

// TODO determine if i need to refer to xmlwriteable things generically
//pub trait XmlWritable {
//    fn write_to<W: std::io::Write>(
//        &self,
//        writer: &mut xml::Writer<W>,
//    ) -> std::io::Result<()>;
//}

/// Representation of MusicXML Score
pub struct Score {
    pub parts: Vec<Part>,
    work_title: String,
    composer: String,
    arranger: String,
    source: String,
}

#[derive(Default)]
pub struct ScoreCreateInfo<'a> {
    pub title: &'a str,
    pub composer: Option<&'a str>,
    pub arranger: Option<&'a str>,
    pub source: Option<&'a str>,
}

impl Score {
    pub fn new(opt: ScoreCreateInfo) -> Self {
        Self {
            parts: Vec::new(),
            work_title: opt.title.to_string(),
            composer: opt.composer.unwrap_or_default().to_string(),
            arranger: opt.arranger.unwrap_or_default().to_string(),
            source: opt.source.unwrap_or_default().to_string(),
        }
    }

    /// Get part by name
    pub fn get_part(&self, name: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.name == name)
    }

    // Mutable version if needed
    pub fn get_part_mut(&mut self, name: &str) -> Option<&mut Part> {
        self.parts.iter_mut().find(|p| p.name == name)
    }

    /// This needs to be used before a score is to be written.
    /// This is because the number of measures across all parts in a score need
    /// to be the same. This function will fill each part with empty measures
    /// as required.
    pub fn finalize(&mut self) {
        let max =
            self.parts.iter().map(|p| p.measures.len()).max().unwrap_or(0);
        for p in &mut self.parts {
            let missing = max - p.measures.len();
            for _ in 0..missing {
                p.empty_measure();
            }
        }
    }

    pub fn write_to<W: std::io::Write>(
        &mut self,
        writer: &mut W,
    ) -> std::io::Result<()> {
        self.finalize();

        let mut w = xml::Writer::new(writer);

        w.raw(r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#)?;
        w.raw(r#"<!DOCTYPE score-partwise PUBLIC"#)?;
        w.raw(r#"    "-//Recordare//DTD MusicXML 4.0 Partwise//EN""#)?;
        w.raw(r#"    "http://www.musicxml.org/dtds/partwise.dtd">"#)?;

        w.open_tag(
            "score-partwise",
            Some(xml::XmlAttributes::new(vec![("version", "4.0")])),
        )
        .unwrap();
        w.open_tag("work", None)?;
        w.text_element("work-title", &self.work_title)?;
        w.close_tag("work")?;
        w.open_tag("identification", None)?;
        w.text_element_with_attrs(
            "creator",
            &self.composer,
            xml::XmlAttributes::new(vec![("type", "composer")]),
        )?;
        w.text_element_with_attrs(
            "creator",
            &self.arranger,
            xml::XmlAttributes::new(vec![("type", "arranger")]),
        )?;
        w.text_element("source", &self.source)?;
        w.close_tag("identification")?;

        w.open_tag("part-list", None)?;
        for part in &self.parts {
            w.open_tag(
                "score-part",
                Some(xml::XmlAttributes::new(vec![("id", &part.id)])),
            )?;
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

        w.close_tag("score-partwise")?;
        Ok(())
    }

    pub fn part<F>(
        &mut self,
        name: &str,
        build: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Part),
    {
        let id = format!("P{}", self.parts.len() + 1);
        let mut part = Part::new(&id, name);
        build(&mut part);
        part.finalize();
        self.parts.push(part);
        Ok(())
    }
}

pub struct Part {
    pub measures: Vec<Measure>,
    pub id: String,
    name: String,
    instrument: Option<MusicXmlInstrument>,

    /// For measures (and notes) to have reference to the most recently defined
    /// <attributes>, this value is updated on measure creation if the measure
    /// contains attributes. This is then rolled forward to each new measure
    /// that does not contain any attributes.
    effective_attributes: Option<Attributes>,
}

impl Part {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            measures: Vec::new(),
            id: id.to_string(),
            name: name.to_string(),
            instrument: None,
            effective_attributes: None,
        }
    }

    pub fn nominal_duration_seconds(&self) -> f64 {
        fn ticks_to_secs(ticks: u32, divisions: u32, bpm: f64) -> f64 {
            let quarter_note_duration = 60.0 / bpm;
            (ticks as f64 / divisions as f64) * quarter_note_duration
        }
        let mut cursor = 0.0 as f64;
        let mut max_cursor = 0.0 as f64;

        let mut tempo_bpm = 120.0 as f64;
        let mut divisions = 480 as u32; // Default assumption, often overridden in first measure

        for measure in &self.measures {
            if let Some(attrs) = &measure.attributes {
                divisions = attrs.divisions;
            }

            for item in &measure.items {
                match item {
                    MeasureItem::Note(note) => {
                        let dur_secs =
                            ticks_to_secs(note.duration, divisions, tempo_bpm);
                        if !note.is_chord {
                            cursor += dur_secs;
                            max_cursor = max_cursor.max(cursor);
                        }
                    }

                    MeasureItem::Forward(fwd) => {
                        let dur_secs =
                            ticks_to_secs(fwd.duration, divisions, tempo_bpm);
                        cursor += dur_secs;
                        max_cursor = max_cursor.max(cursor);
                    }

                    MeasureItem::Backup(bak) => {
                        let dur_secs =
                            ticks_to_secs(bak.duration, divisions, tempo_bpm);
                        cursor -= dur_secs;
                        // Backup does not affect max_cursor
                    }

                    MeasureItem::Direction(dir) => {
                        if let DirectionType::Metronome { per_minute, .. } =
                            &dir.kind
                        {
                            tempo_bpm = *per_minute as f64;
                        }
                    }

                    _ => {}
                }
            }
        }

        max_cursor
    }

    /// For tied notes (and in the future, other types of notations) to be able
    /// to span across measures, a post-processing step is required to place
    /// the correct stop/continue tags on ties based on context that is not
    /// available to a single note or measure.
    pub fn finalize(&mut self) {
        let mut previous_tied_pitch: Option<Pitch> = None;
        let mut previous_note: Option<&mut Note> = None;

        for measure in &mut self.measures {
            for item in &mut measure.items {
                match item {
                    MeasureItem::Note(note) => {
                        match (&note.pitch, note.tie.clone()) {
                            (Some(pitch), Some(StartStop::Start)) => {
                                // Begin a new tie
                                previous_tied_pitch = Some(pitch.clone());
                                previous_note = Some(note);
                            }

                            (Some(pitch), None) => {
                                // If this note continues a tie, mark it as stop
                                if let Some(prev_pitch) = &previous_tied_pitch {
                                    if pitch.to_semitone()
                                        == prev_pitch.to_semitone()
                                    {
                                        if let Some(prev_note) =
                                            previous_note.as_mut()
                                        {
                                            prev_note.tie =
                                                Some(StartStop::Start);
                                        }
                                        note.tie = Some(StartStop::Stop);
                                        previous_tied_pitch = None;
                                        previous_note = None;
                                    }
                                }
                            }

                            (Some(pitch), Some(StartStop::Stop)) => {
                                if let Some(prev_pitch) = &previous_tied_pitch {
                                    if pitch.to_semitone()
                                        == prev_pitch.to_semitone()
                                    {
                                        if let Some(prev_note) =
                                            previous_note.as_mut()
                                        {
                                            prev_note.tie =
                                                Some(StartStop::Start);
                                        }
                                    }
                                }
                                previous_tied_pitch = None;
                                previous_note = None;
                            }

                            _ => {
                                previous_tied_pitch = None;
                                previous_note = None;
                            }
                        }
                    }

                    _ => {
                        // Reset state on non-note items
                        previous_tied_pitch = None;
                        previous_note = None;
                    }
                }
            }
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag(
            "part",
            Some(xml::XmlAttributes::new(vec![("id", &self.id)])),
        )?;
        for m in &self.measures {
            m.write_to(writer)?;
        }
        writer.close_tag("part")?;
        Ok(())
    }

    pub fn measure<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Measure),
    {
        let mut m = Measure::new(MeasureCreateInfo {
            measure_number: self.measures.len() + 1,
            attributes: None,
            effective_attributes: self.effective_attributes.clone(),
        });

        f(&mut m);

        // After measure is created by user, save attributes to part.
        // This assumes that measures are added to the part in order.
        // TODO i think removing this part-level state is good but i need to
        // find another solution for parts to have attribute awareness
        if let Some(attr) = &m.attributes {
            self.effective_attributes = Some(attr.clone());
        }

        self.measures.push(m);
    }

    // TODO allow optional attributes so empty measures can be prepended to part
    pub fn empty_measure(&mut self) {
        let attrs = self
            .get_current_attr()
            .expect("Cannot add empty measure: no previous attributes found");

        // Calculate ticks for full measure duration
        let beat_unit_ratio = 4.0 / attrs.time_beat_type as f32;
        let beat_ticks = (attrs.divisions as f32 * beat_unit_ratio) as u32;
        let total_divisions = attrs.time_beats as u32 * beat_ticks;

        let rest = Note::new(NoteCreateInfo {
            pitch: None,
            kind: NoteType::Whole, // not used
            is_measure_rest: true,
            duration_override: Some(total_divisions),
            ..NoteCreateInfo::default()
        });

        let number = self.measures.len() + 1;

        let mut measure = Measure::new(MeasureCreateInfo {
            measure_number: number,
            attributes: None,
            effective_attributes: self.effective_attributes.clone(),
        });

        measure.item(MeasureItem::Note(rest));
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

    /// Add a MusicXML instrument (combined <midi-instrument> and
    /// <score-instrument>). This instrument is a suggestion to external  
    /// programs reading the XML file. The actual playback depends on the user
    /// of the file. For lyra related audio rendering, use the instruments
    /// defined in the render layer.
    ///
    /// Many options in both instrument elements are missing from this
    /// convenience function.
    pub fn instrument(
        &mut self,
        ci: MusicXmlInstrumentCreateInfo,
        //name: &str,
        //midi_program: Option<u8>,
        //sound: Option<String>,
    ) {
        // TODO only supports one instrument per part currently
        self.instrument =
            Some(MusicXmlInstrument::new(MusicXmlInstrumentCreateInfo {
                part_id: ci.part_id,
                instrument_id: ci.instrument_id,
                name: ci.name,
                midi_program: ci.midi_program,
                sound: ci.sound,
            }));
    }
}

/// All modes allowed in <mode> from the MusicXML spec
#[derive(Clone)]
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

impl std::str::FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "major" => Ok(Mode::Major),
            "minor" => Ok(Mode::Minor),
            "dorian" => Ok(Mode::Dorian),
            "phrygian" => Ok(Mode::Phrygian),
            "lydian" => Ok(Mode::Lydian),
            "mixolydian" => Ok(Mode::Mixolydian),
            "aeolian" => Ok(Mode::Aeolian),
            "ionian" => Ok(Mode::Ionian),
            "locrian" => Ok(Mode::Locrian),
            "none" => Ok(Mode::None),
            other => Err(format!("Unknown mode: '{}'", other)),
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

#[derive(Clone, PartialEq)]
pub enum Clef {
    Treble,
    Bass,
    Alto,
    Soprano,
    Tenor,
    Percussion,
}

impl Clef {
    pub fn to_sign(&self) -> String {
        match self {
            Self::Soprano | Self::Alto => "C".to_string(),
            Self::Tenor | Self::Treble => "G".to_string(),
            Self::Bass => "F".to_string(),
            Self::Percussion => "percussion".to_string(),
        }
    }

    pub fn to_line(&self) -> u8 {
        match self {
            Self::Soprano => 1,
            Self::Treble | Self::Percussion => 2,
            Self::Alto => 3,
            Self::Tenor | Self::Bass => 4,
        }
    }
}

impl std::str::FromStr for Clef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "treble" => Ok(Clef::Treble),
            "bass" => Ok(Clef::Bass),
            "alto" => Ok(Clef::Alto),
            "soprano" => Ok(Clef::Soprano),
            "tenor" => Ok(Clef::Tenor),
            "percussion" => Ok(Clef::Percussion),
            other => Err(format!("Unknown clef: '{}'", other)),
        }
    }
}

pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl std::str::FromStr for TimeSignature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        assert!(parts.len() == 2, "Improperly formatted time signature found");

        let numerator = parts[0]
            .parse::<u8>()
            .map_err(|_| format!("Invalid numerator: '{}'", parts[0]))?;
        let denominator = parts[1]
            .parse::<u8>()
            .map_err(|_| format!("Invalid denominator: '{}'", parts[1]))?;

        Ok(TimeSignature { numerator, denominator })
    }
}

pub struct AttributesCreateInfo<'a> {
    pub key_name: &'a str,
    pub key_mode: &'a str,
    pub time_sig: &'a str,
    pub clefs: Vec<&'a str>,
}

impl Default for AttributesCreateInfo<'_> {
    fn default() -> Self {
        Self {
            key_name: "C",
            key_mode: "major",
            time_sig: "4/4",
            clefs: vec!["treble"],
        }
    }
}

#[derive(Clone)]
pub struct StaffDetails {
    staff_lines: u8,
}

#[derive(Clone)]
pub struct Attributes {
    pub divisions: u32,
    pub key_fifths: i8, // 0 = C major, -1 = F major, 1 = G major
    pub key_mode: Mode,
    pub time_beats: u8,     // numerator
    pub time_beat_type: u8, // denominator
    pub clefs: Vec<Clef>,
    pub staves: Option<usize>,
    pub staff_details: Option<StaffDetails>,
}

impl Attributes {
    pub fn new(ci: &AttributesCreateInfo) -> Self {
        let mut staves = None;
        if ci.clefs.len() > 1 {
            staves = Some(ci.clefs.len());
        }
        let clefs = ci.clefs.iter().map(|c| c.parse().unwrap()).collect();
        let time_sig: TimeSignature = ci.time_sig.parse().unwrap();
        let mut staff_details = None;
        if ci.clefs.len() == 1 && ci.clefs[0] == "percussion" {
            staff_details = Some(StaffDetails { staff_lines: 1 });
        }

        Self {
            // TODO divisions are hard coded to be 480. Add more flexibility
            // so its globally applied to score
            divisions: 480,
            key_fifths: key_fifths_from_name(&ci.key_name),
            key_mode: ci.key_mode.parse().unwrap(),
            time_beats: time_sig.numerator,
            time_beat_type: time_sig.denominator,
            staves,
            clefs,
            staff_details,
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        let w = writer;
        w.open_tag("attributes", None)?;
        w.text_element("divisions", &self.divisions.to_string())?;

        w.open_tag("key", None)?;
        w.text_element("fifths", &self.key_fifths.to_string())?;
        w.text_element("mode", &self.key_mode.to_string())?;
        w.close_tag("key")?;

        w.open_tag("time", None)?;
        w.text_element("beats", &self.time_beats.to_string())?;
        w.text_element("beat-type", &self.time_beat_type.to_string())?;
        w.close_tag("time")?;

        for (index, clef) in self.clefs.iter().enumerate() {
            w.open_tag(
                "clef",
                Some(xml::XmlAttributes::new(vec![(
                    "number",
                    &(index + 1).to_string(),
                )])),
            )?;
            w.text_element("sign", &clef.to_sign().to_string())?;
            w.text_element("line", &clef.to_line().to_string())?;
            w.close_tag("clef")?;
        }

        // TODO the <staves> element throws an error in MuseScore.
        // Check if this is my impl issue or musescores.
        if let Some(staves) = &self.staves {
            w.text_element("staves", &staves.to_string())?;
        }

        if let Some(details) = &self.staff_details {
            w.open_tag("staff-details", None)?;
            w.text_element("staff-lines", &details.staff_lines.to_string())?;
            w.close_tag("staff-details")?;
        }

        w.close_tag("attributes")?;
        Ok(())
    }
}

/// Any MusicXML element that can be placed at the measure level
pub enum MeasureItem {
    Note(Note),
    Direction(Direction),
    Backup(Backup),
    Forward(Forward),
    Barline(Barline),
}

// https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/barline/
pub struct Barline {
    bar_style: Option<BarStyle>,
    ending: Option<Ending>,
    repeat: Option<Repeat>,
    //fermata: Option<Fermata>,
    //coda: Option<Coda>,
    //segno: Option<Segno>,
    //wavy_line: Option<WavyLine>,
    //footnote: Option<Footnote>,

    // Attributes
    //coda,
    //divisions,
    //id,
    //location,
    //segno
}

// TODO bar style elements can contain a color attribute too. This would require
// a bar style struct. This enum would become BarStyleType
pub enum BarStyle {
    Dashed,
    Dotted,
    Heavy,
    HeavyHeavy,
    HeavyLight,
    LightHeavy,
    LightLight,
    None,
    Regular,
    Short,
    Tick,
}

impl BarStyle {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
            Self::Heavy => "heavy",
            Self::HeavyHeavy => "heavy-heavy",
            Self::HeavyLight => "heavy-light",
            Self::LightHeavy => "light-heavy",
            Self::LightLight => "light-light",
            Self::None => "none",
            Self::Regular => "regular",
            Self::Short => "short",
            Self::Tick => "tick",
        }
    }
}

// https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/ending/
pub struct Ending {
    content: String, // Usually 1. or 1.,2.

    number: u8,   // ending number
    kind: String, // Start, Stop, Discontinue
}

// https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/repeat/
pub struct Repeat {
    direction: String, // "backward" or "forward"
}

/// Representation of <backup> element. Moves the time cursor back a set
/// duration in ticks.
pub struct Backup {
    pub duration: u32,
    footnote: Option<String>, // TODO implement
    level: Option<String>,    // TODO implement
}

impl Backup {
    /// Takes a vec of note types (durations) and creates a backup of equivalent
    /// duration
    pub fn from_note_types(kinds: &[NoteType], divisions: u32) -> Self {
        // TODO note duration cannot currently include dots or time mods
        // as seen in the to_duration() fn
        let duration =
            kinds.iter().map(|t| t.to_duration(divisions, None, None)).sum();
        Self { duration, footnote: None, level: None }
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

/// Representation of <forward> element. Moves time cursor forward a certain
/// duration measured in ticks.
pub struct Forward {
    pub duration: u32,
    footnote: Option<String>,
    level: Option<String>,
    staff: Option<u8>,
}

/// Representation of <measure>. Each measure has an optional attributes
/// element which sets things like time signature or key for all measures
/// proceeding it.
pub struct Measure {
    pub items: Vec<MeasureItem>,
    number: usize,
    pub attributes: Option<Attributes>,

    /// This value is cloned from the parent part of the measure. This is used
    /// in measures that do not define new attributes. This is separate from
    /// attributes because it should not be written to XML.
    pub effective_attributes: Option<Attributes>,
}

pub struct MeasureCreateInfo {
    measure_number: usize,
    attributes: Option<Attributes>,
    effective_attributes: Option<Attributes>,
}

impl Measure {
    pub fn new(ci: MeasureCreateInfo) -> Self {
        Self {
            number: ci.measure_number,
            items: Vec::new(),
            attributes: ci.attributes,
            effective_attributes: ci.effective_attributes,
        }
    }

    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        writer.open_tag(
            "measure",
            Some(xml::XmlAttributes::new(vec![(
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
                MeasureItem::Barline(_) => panic!("Not implemented"),
            }
        }
        writer.close_tag("measure")?;
        Ok(())
    }

    /// The most generalized way to append to a measure. Functions that
    /// add any measure elements to a part use this internally.
    pub fn item(&mut self, item: MeasureItem) {
        self.items.push(item);
    }

    /// Update the measure's attributes. Overwrites effective attributes.
    pub fn attributes(&mut self, ci: &AttributesCreateInfo) {
        assert!(
            self.items.len() == 0,
            "Attributes must be set before items are added to it"
        );
        let attr = Attributes::new(ci);
        self.attributes = Some(attr.clone());
        self.effective_attributes = Some(attr);
    }

    // TODO consider if the user can add a staff distinction or placement.
    // This fn is intended to prioritize convenience over customizability.
    // Add to beat_unit type safety to allow for dotted units and more clarity
    // to the fn user.
    pub fn metronome(&mut self, beat_unit: &str, per_minute: u32) {
        self.item(MeasureItem::Direction(Direction {
            kind: DirectionType::Metronome {
                beat_unit: beat_unit.to_string(),
                per_minute,
            },
            placement: Some("above".to_string()),
            staff: None,
        }))
    }

    /// Append a dynamic direction to a measure to items list
    pub fn dynamics(&mut self, dynamics: &str) {
        self.item(MeasureItem::Direction(Direction {
            kind: DirectionType::Dynamics(Dynamics::from_str(dynamics)),
            placement: Some("below".to_string()),
            staff: None,
        }));
    }

    pub fn note_repeat(&mut self, note_str: &str, count: u32) {
        for _ in 0..count {
            self.note(note_str);
        }
    }

    /// Convenience function to add a note to a measure.
    /// Parses notes from custom DSL format:
    /// pitch:duration
    /// ie. "C4:h." -> C note, 4th octave, dotted half note
    /// See NoteType::from_char() for all duration chars
    pub fn note(&mut self, note_str: &str) {
        let parts: Vec<&str> = note_str.split(':').collect();
        assert!(parts.len() == 2, "note requires a pitch:duration notation");

        // initial note from string parse
        let mut note = Note::new(note_str.parse().unwrap());

        // TODO Automatic staff placement for multi staff parts
        // seems to be somewhat context dependent near middle C.
        // I need to decide if a more manual method in this API
        // is required.

        // TODO this might be too strict since attr is optional in MusicXML.
        // In lyra it is almost required because I tend not to write music
        // without attributes.
        let attr = self.attributes.as_ref().unwrap_or_else(|| {
            self.effective_attributes
                .as_ref()
                .expect("Effective attributes must be set")
        });

        // Check if the note should be unpitched. If so, convert the pitch
        // to an unpitched with the same tone and octave (ignoring alter).
        if attr.clefs[0] == Clef::Percussion {
            if let Some(pitch) = note.pitch {
                note.unpitched = Some(Unpitched {
                    display_step: pitch.step,
                    display_octave: pitch.octave,
                });
                note.pitch = None;
            }
        }

        // Support only 1 or 2 staves for now. I'm not sure if more than 2
        // staves in a part is common enough to change.
        if attr.staves == Some(2) {
            // The only multi-staff case supported is a treble + bass combo
            if let [Clef::Treble, Clef::Bass] = attr.clefs.as_slice() {
                if let Some(pitch) = &note.pitch {
                    if pitch.to_semitone() >= 60 {
                        note.staff = Some(1);
                    } else {
                        note.staff = Some(2);
                    }
                }
            }
        }
        self.item(MeasureItem::Note(note));
    }

    /// Convenience function to add a rest to a measure.
    /// Parses rests from custom DSL format specifying duration
    /// ie. "h." -> dotted half rest
    /// See NoteType::from_char() for all duration chars
    pub fn rest(&mut self, rest: &str) {
        let parts: Vec<&str> = rest.split(':').collect();
        assert!(parts.len() == 1, "rest requires a duration notation");
        self.item(MeasureItem::Note(Note::new(rest.parse().unwrap())));
    }

    /// Convenience function to add a chord to a measure.
    /// Parses chord from custom DSL format specifying root:quality:duration
    /// ie. "maj:C4:h." -> Cmaj triad with dotted half note duration
    pub fn chord(&mut self, chord_str: &str) {
        let mut parts = chord_str.splitn(2, ":");
        let quality = parts.next().unwrap().parse().unwrap();
        let note = Note::new(parts.next().unwrap().parse().unwrap());
        let chord = Chord::new(note.pitch.unwrap(), quality, None);

        // TODO refactor to_notes() API. This is hard coded for now. This means
        // chord doesnt work for multi staff parts or multi voice parts.
        let notes = chord.to_notes(note.kind, 480, None, None);

        for n in notes {
            self.items.push(MeasureItem::Note(n));
        }
    }
}

pub struct MusicXmlInstrumentCreateInfo {
    pub part_id: String,
    pub instrument_id: u32,
    pub name: String,
    pub midi_program: Option<u8>,
    pub sound: Option<String>,
}

// TODO this name is tentative
pub struct MusicXmlInstrument {
    pub midi: MidiInstrument,
    pub score: ScoreInstrument,
}

impl MusicXmlInstrument {
    pub fn new(ci: MusicXmlInstrumentCreateInfo) -> Self {
        let id = format!("{}-I{}", &ci.part_id, ci.instrument_id);

        MusicXmlInstrument {
            midi: MidiInstrument {
                id: id.clone(),
                program: ci.midi_program,
                ..MidiInstrument::default()
            },
            score: ScoreInstrument {
                id,
                name: ci.name.into(),
                sound: ci.sound,
                ..ScoreInstrument::default()
            },
        }
    }
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
            Some(xml::XmlAttributes::new(vec![("id", &self.id)])),
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
            Some(xml::XmlAttributes::new(vec![("id", &self.id)])),
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
    pub fn write_to<W: std::io::Write>(
        &self,
        writer: &mut xml::Writer<W>,
    ) -> std::io::Result<()> {
        let attrs = self
            .placement
            .as_ref()
            .map(|place| vec![("placement", place.as_str())])
            .unwrap_or_default();

        writer.open_tag("direction", Some(xml::XmlAttributes::new(attrs)))?;
        writer.open_tag("direction-type", None)?;

        match &self.kind {
            DirectionType::Words(text) => {
                writer.text_element("words", text)?;
            }
            DirectionType::Metronome { beat_unit, per_minute } => {
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

    // TODO implement std::std::FromStr instead
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

    pub fn normalized_velocity(&self) -> f64 {
        match self {
            Dynamics::PPP => 16.0 / 127.0,
            Dynamics::PP => 33.0 / 127.0,
            Dynamics::P => 49.0 / 127.0,
            Dynamics::MP => 64.0 / 127.0,
            Dynamics::MF => 80.0 / 127.0,
            Dynamics::F => 96.0 / 127.0,
            Dynamics::FF => 112.0 / 127.0,
            Dynamics::FFF => 127.0 / 127.0,
        }
    }
}

/// MusicXML tied-type used for <tied> elements. This is for notations, not
/// sound direction.
pub enum Tied {
    Start,
    Stop,
    Continue,
    LetRing,
}

impl Tied {
    pub fn to_string(&self) -> String {
        match self {
            Self::Start => "start".to_string(),
            Self::Stop => "stop".to_string(),
            Self::Continue => "continue".to_string(),
            Self::LetRing => "let-ring".to_string(),
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
#[derive(Clone)]
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
    Tied(Tied),
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
                Some(xml::XmlAttributes::new(vec![(
                    "type",
                    &t.kind.to_string(),
                )])),
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

/// To represent things like triplets
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

pub struct Unpitched {
    display_step: NaturalTone,
    display_octave: i8,
}

pub struct Note {
    kind: NoteType,
    pub pitch: Option<Pitch>,
    pub unpitched: Option<Unpitched>,
    pub is_chord: bool,
    pub duration: u32,
    staff: Option<u8>,
    voice: Option<u8>,
    time_mod: Option<TimeModification>,
    notations: Option<Notations>,
    dots: Option<u8>,
    pub tie: Option<StartStop>,
    is_measure_rest: bool,
}

pub struct NoteCreateInfo {
    pub kind: NoteType,
    pub divisions: u32,
    pub is_chord: bool,
    pub pitch: Option<Pitch>,
    pub unpitched: Option<Unpitched>,
    pub staff: Option<u8>,
    pub voice: Option<u8>,
    pub time_mod: Option<TimeModification>,
    pub notations: Option<Notations>,
    pub dots: Option<u8>,
    pub tie: Option<StartStop>,
    pub is_measure_rest: bool,

    // TODO this is to handle the case of measure rests. Measure rests should
    // ignore the duration derived from note type and other elements, measure
    // rests will calculate the exact duration in ticks based on time
    // signature. Maybe there is a cleaner implementation
    pub duration_override: Option<u32>,
}

impl Default for NoteCreateInfo {
    fn default() -> Self {
        Self {
            kind: NoteType::Quarter,
            pitch: None,
            unpitched: None,
            divisions: 480,
            is_chord: false,
            staff: None,
            voice: None,
            time_mod: None,
            notations: None,
            dots: None,
            tie: None,
            is_measure_rest: false,
            duration_override: None,
        }
    }
}

impl std::str::FromStr for NoteCreateInfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() > 2 {
            return Err("Expected format like C#4:h.".to_string());
        }
        let (pitch_opt, raw_duration_str) = match parts.len() {
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

        // Handle tie symbol (~) anywhere in the duration part.
        // The stop tie types will be placed in part finalization
        let tie = if raw_duration_str.contains('~') {
            Some(StartStop::Start)
        } else {
            None
        };
        let duration_str = raw_duration_str.replace('~', "");

        // Parse duration and dots
        let mut chars = duration_str.chars();
        let type_char = chars.next().ok_or("Missing note type")?;
        let kind = NoteType::from_char(type_char);
        let dot_count = chars.filter(|&c| c == '.').count();

        Ok(NoteCreateInfo {
            pitch: pitch_opt,
            kind,
            dots: if dot_count > 0 { Some(dot_count as u8) } else { None },
            tie,
            ..NoteCreateInfo::default()
        })
    }
}

impl Note {
    pub fn new(opt: NoteCreateInfo) -> Self {
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
            unpitched: opt.unpitched,
            is_chord: opt.is_chord,
            duration,
            staff: opt.staff,
            voice: opt.voice,
            time_mod: opt.time_mod,
            notations: opt.notations,
            dots: opt.dots,
            tie: opt.tie,
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

        // TODO there should never be the case where there is a pitch and
        // unpitched in the same note element. Currently left to the user to
        // not do this.
        if let Some(pitch) = &self.pitch {
            pitch.write_to(writer)?;
        } else if let Some(unpitched) = &self.unpitched {
            writer.open_tag("unpitched", None)?;
            writer.text_element(
                "display-step",
                &unpitched.display_step.to_char().to_string(),
            )?;
            writer.text_element(
                "display-octave",
                &unpitched.display_octave.to_string(),
            )?;
            writer.close_tag("unpitched")?;
        } else {
            let rest_attrs = if self.is_measure_rest {
                Some(xml::XmlAttributes::new(vec![("measure", "yes")]))
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

        if let Some(tie) = &self.tie {
            writer.self_closing_tag(
                "tie",
                Some(xml::XmlAttributes::new(vec![("type", &tie.to_string())])),
            )?;
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
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
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

        // TODO this can be generalized
        let dotted = match dots.unwrap_or(0) {
            0 => base,
            1 => base + (base / 2),
            2 => base + (base / 2) + (base / 4),
            3 => base + (base / 2) + (base / 4) + (base / 8),
            _ => panic!("Unsupported number of dots: >3"),
        };

        // TODO put this functionality inside time mod struct
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
        assert!(semitone > 0, "Semitone out of midi range");
        semitone as u8
    }

    /// From a semitone value relative to C-1 (MIDI value of 0)
    pub fn from_semitone(semitone: u8, prefer_flat: bool) -> Self {
        assert!(semitone <= 127, "semitone out of MIDI range");

        let octave = (semitone / 12) as i8 - 1;
        let semitone_in_octave = semitone % 12;

        if prefer_flat {
            match semitone_in_octave {
                0 => Pitch { step: NaturalTone::C, octave, alter: None },
                1 => Pitch { step: NaturalTone::D, octave, alter: Some(-1) }, /* D♭ */
                2 => Pitch { step: NaturalTone::D, octave, alter: None },
                3 => Pitch { step: NaturalTone::E, octave, alter: Some(-1) }, /* E♭ */
                4 => Pitch { step: NaturalTone::E, octave, alter: None },
                5 => Pitch { step: NaturalTone::F, octave, alter: None },
                6 => Pitch { step: NaturalTone::G, octave, alter: Some(-1) }, /* G♭ */
                7 => Pitch { step: NaturalTone::G, octave, alter: None },
                8 => Pitch { step: NaturalTone::A, octave, alter: Some(-1) }, /* A♭ */
                9 => Pitch { step: NaturalTone::A, octave, alter: None },
                10 => Pitch { step: NaturalTone::B, octave, alter: Some(-1) }, /* B♭ */
                11 => Pitch { step: NaturalTone::B, octave, alter: None },
                _ => unreachable!("invalid semitone in octave"),
            }
        } else {
            match semitone_in_octave {
                0 => Pitch { step: NaturalTone::C, octave, alter: None },
                1 => Pitch { step: NaturalTone::C, octave, alter: Some(1) }, /* C♯ */
                2 => Pitch { step: NaturalTone::D, octave, alter: None },
                3 => Pitch { step: NaturalTone::D, octave, alter: Some(1) }, /* D♯ */
                4 => Pitch { step: NaturalTone::E, octave, alter: None },
                5 => Pitch { step: NaturalTone::F, octave, alter: None },
                6 => Pitch { step: NaturalTone::F, octave, alter: Some(1) }, /* F♯ */
                7 => Pitch { step: NaturalTone::G, octave, alter: None },
                8 => Pitch { step: NaturalTone::G, octave, alter: Some(1) }, /* G♯ */
                9 => Pitch { step: NaturalTone::A, octave, alter: None },
                10 => Pitch { step: NaturalTone::A, octave, alter: Some(1) }, /* A♯ */
                11 => Pitch { step: NaturalTone::B, octave, alter: None },
                _ => unreachable!("invalid semitone in octave"),
            }
        }
    }

    pub fn to_frequency(&self) -> f64 {
        let semitone_offset = self.to_semitone();
        440.0 * 2f64.powf((semitone_offset as f64 - 69.0) / 12.0)
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

        Ok(Pitch { step, alter, octave })
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
        Self { root, pitches, quality, transform }
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
            notes.push(Note::new(NoteCreateInfo {
                pitch: Some(p.clone()),
                is_chord,
                kind: note_kind.clone(),
                divisions,
                staff,
                voice,
                ..NoteCreateInfo::default()
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

impl std::str::FromStr for ChordQuality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "maj" | "major" => Ok(Self::Major),
            "min" | "minor" => Ok(Self::Minor),
            "dim" | "diminished" => Ok(Self::Diminished),
            "aug" | "augmented" => Ok(Self::Augmented),
            "sus2" => Ok(Self::Sus2),
            "sus4" => Ok(Self::Sus4),
            "maj7" | "major7" => Ok(Self::Major7),
            "min7" | "minor7" => Ok(Self::Minor7),
            "min7b5" | "minor7b5" => Ok(Self::Minor7b5),
            "minmaj7" | "minormajor7" => Ok(Self::MinorMajor7),
            "maj6" | "major6" => Ok(Self::Major6),
            "min6" | "minor6" => Ok(Self::Minor6),
            "7" | "dom7" | "dominant7" => Ok(Self::Dominant7),
            "maj9" | "major9" => Ok(Self::Major9),
            "min9" | "minor9" => Ok(Self::Minor9),
            "dom9" | "dominant9" => Ok(Self::Dominant9),
            other => Err(format!("Unknown chord quality: '{}'", other)),
        }
    }
}

pub struct Scale;
pub struct Voice;

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
