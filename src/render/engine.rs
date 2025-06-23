use std::collections::{HashMap, HashSet, VecDeque};

use hound::{SampleFormat, WavSpec, WavWriter};

use super::effect::AudioEffect;
use super::instrument::Instrument;
use super::processor::AudioBuffer;
use crate::compose::Part;

/// The top level of the rendering layer
pub struct Engine {
    pub sample_rate: u32,
    pub block_size: u32,
    pub node_graph: Graph,
}

impl Engine {
    pub fn render(&mut self, output_path: &str) {
        let graph = &self.node_graph;
        let order = graph.topological_sort();
        let mut block_time = 0.0;

        let block_size = self.block_size as usize;
        let sample_rate = self.sample_rate as f64;
        let block_duration = block_size as f64 / sample_rate;

        // Global output buffer (may stream to WAV file in chunks instead)
        let mut output_writer = WavWriter::create(
            output_path,
            WavSpec {
                channels: 2,
                sample_rate: self.sample_rate,
                bits_per_sample: 32,
                sample_format: SampleFormat::Float,
            },
        );

        let mut buffer_map: HashMap<NodeId, AudioBuffer> = HashMap::new();

        while !self.finished_rendering(block_time) {
            // Clear buffer map for new block
            buffer_map.clear();

            for node_id in order {
                let node = graph.get_node(node_id);
                let input_buffers = graph
                    .inputs(node_id)
                    .iter()
                    .filter_map(|input_id| buffer_map.get(input_id))
                    .cloned()
                    .collect::<Vec<_>>();

                // Get a clean output buffer from pool
                let mut out_buffer = self.buffer_pool.get();
                out_buffer.resize(block_size);

                // Process the node
                match &node.kind {
                    NodeKind::Track { source, driver } => {
                        // Feed current clock time + block_size
                        source.process_block(
                            driver,
                            block_time,
                            block_duration,
                            &mut out_buffer,
                        );
                    }
                    NodeKind::Effect { effect } => {
                        if let Some(input) = input_buffers.first() {
                            out_buffer.copy_from(input);
                            effect.process(&mut out_buffer, sample_rate as u32);
                        }
                    }
                    NodeKind::Send { amount } => {
                        if let Some(input) = input_buffers.first() {
                            for i in 0..block_size {
                                let sample = input.get(i) * *amount as f32;
                                out_buffer.set(i, sample);
                            }
                        }
                    }
                    NodeKind::Bus => {
                        // Sum all inputs
                        for input in &input_buffers {
                            out_buffer.add(input);
                        }
                    }
                    NodeKind::Output => {
                        for input in &input_buffers {
                            out_buffer.add(input);
                        }
                        output_writer.write_block(&out_buffer);
                    }
                }

                buffer_map.insert(*node_id, out_buffer);
            }

            block_time += block_duration;

            // Return all buffers to pool
            for (_, buf) in buffer_map.drain() {
                self.buffer_pool.release(buf);
            }
        }

        output_writer.finalize();
    }
}

pub struct Clock {
    pub sample_rate: u32,
    pub sample_counter: u64,
}

impl Clock {
    pub fn advance(&mut self, block_size: usize) {
        self.sample_counter += block_size as u64;
    }

    pub fn time(&self) -> f64 {
        self.sample_counter as f64 / self.sample_rate as f64
    }

    pub fn sample(&self) -> u64 {
        self.sample_counter
    }

    // Optional, if working in beats
    pub fn beat(&self, bpm: f64) -> f64 {
        self.time() * (bpm / 60.0)
    }
}

/// DAG audio node graph
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub type NodeId = u32;

// From => To
pub type Edge = (NodeId, NodeId);

impl Graph {
    pub fn new() -> Self {
        Graph { nodes: Vec::new(), edges: Vec::new() }
    }

    /// Add a node to the graph and return its NodeId
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node { id, kind });
        id
    }

    /// Connect two nodes (from → to)
    pub fn connect(&mut self, from: NodeId, to: NodeId) {
        self.edges.push((from, to));
    }

    /// Disconnect two nodes (from → to)
    pub fn disconnect(&mut self, from: NodeId, to: NodeId) {
        self.edges.retain(|(a, b)| *a != from || *b != to);
    }

    /// Get a reference to a node by its ID
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get a mutable reference to a node by its ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// List all edges from a given node
    pub fn outputs(&self, from: NodeId) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter_map(|(a, b)| if *a == from { Some(*b) } else { None })
            .collect()
    }

    /// List all edges into a given node
    pub fn inputs(&self, to: NodeId) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter_map(|(a, b)| if *b == to { Some(*a) } else { None })
            .collect()
    }

    /// Count how many inputs a node has
    pub fn input_count(&self, id: NodeId) -> usize {
        self.inputs(id).len()
    }

    /// Count how many outputs a node has
    pub fn output_count(&self, id: NodeId) -> usize {
        self.outputs(id).len()
    }

    pub fn topological_sort(&self) -> Vec<NodeId> {
        // Count incoming edges per node
        let mut in_degree: HashMap<NodeId, usize> =
            self.nodes.iter().map(|n| (n.id, 0)).collect();

        for &(_, to) in &self.edges {
            *in_degree.entry(to).or_insert(0) += 1;
        }

        // Start with nodes that have no incoming edges
        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(current) = queue.pop_front() {
            result.push(current);

            for &(from, to) in &self.edges {
                if from == current {
                    if let Some(degree) = in_degree.get_mut(&to) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(to);
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.nodes.len() {
            panic!("Graph has cycles; cannot perform topological sort");
        }

        result
    }
}

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
