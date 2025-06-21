use super::effect::AudioEffect;
use super::instrument::Instrument;
use super::processor::AudioBuffer;
use crate::compose::Part;

/// The top level of the rendering layer
pub struct Engine {
    pub graph: Graph,
    pub context: Context,
    pub block_buffer: AudioBuffer,
}

pub struct Context {
    pub sample_rate: u32,
    pub block_size: usize,
}

/// DAG audio node graph
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub type NodeId = u32;

// From => To
pub type Edge = (NodeId, NodeId);

pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
}

pub enum NodeKind {
    Track { source: SoundSource, driver: EventDriver },
    Effect { effect: Box<dyn AudioEffect> },
    Bus,
    Send { amount: f64 },
    Output,
}

/// Component of a track that dictates when audio events occur
pub enum EventDriver {
    MusicXmlPart(Part),
}

/// Component of a track that defines how events are converted to samples
pub enum SoundSource {
    Instrument(Instrument),
}
