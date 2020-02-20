use crate::operator::Operator;
use std::collections::HashMap;

/// The contents of this NodeId type cannot
/// ever be equal to "NIL_NODE_ID", because
/// that is essentially null
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    #[inline]
    pub fn maybe(self) -> MaybeNodeId {
        MaybeNodeId(self.0)
    }
}

const NIL_NODE_ID: u32 = std::u32::MAX;

/// Just an internal representation to save
/// space instead of using an option(and not introduce
/// a null value on NodeIds)
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct MaybeNodeId(u32);

impl MaybeNodeId {
    #[inline]
    fn none() -> MaybeNodeId {
        MaybeNodeId(NIL_NODE_ID)
    }

    #[inline]
    fn from_option(option: Option<NodeId>) -> MaybeNodeId {
        match option {
            Some(v) => MaybeNodeId(v.0),
            None => MaybeNodeId::none(),
        }
    }

    #[inline]
    fn get(self) -> Option<NodeId> {
        if self.0 == NIL_NODE_ID {
            None
        }else{
            Some(NodeId(self.0))
        }
    }
}

const MAX_INPUTS: usize = 3;

/// TODO: Make this a property of the
/// datastructure and not something that the user
/// has to keep in mind on their own.
///
/// Please note that the ID:s of nodes
/// will change when you remove a node.
/// Therefore, to avoid confuzzlement,
/// please make sure that you do not delete
/// any node while holding onto a Node
/// of your own. Make sure to drop all Nodes
/// before deleting anything.
#[derive(Debug)]
pub struct Synth {
    // The "nodes" in the synth.
    nodes: Vec<Node>,
    // TODO: Allow for static node id's, i.e.
    // node ids that never change and always
    // point to the same node. Of course,
    // they may not point to any node if the
    // node was removed, but they will never point
    // to the wrong node.
    // These would then be what was handed out to
    // external parties if they had to keep track
    // of some node.
    static_node_id_map: HashMap<u32, NodeId>,
    static_node_id_ctr: u32,

    // Probe stuff
    probes: HashMap<NodeId, Probe>,
    probe_id_map: HashMap<ProbeId, NodeId>,
    probe_id_ctr: u32,
}

impl Synth {
    pub fn new() -> Synth {
        Synth {
            nodes: Vec::new(),
            static_node_id_map: HashMap::new(),
            static_node_id_ctr: 0,

            probes: HashMap::new(),
            probe_id_map: HashMap::new(),
            probe_id_ctr: 0,
        }
    }

    pub fn run(&mut self, buffer: &mut Vec<f32>, dt_per_sample: f32) {
        buffer.clear();

        let mut input_buffer = [0f32; MAX_INPUTS];

        // Just to let the borrow checker know that
        // I am not borrowing self in a wierd say
        let mut nodes = self.nodes.iter_mut();
        let probes = &self.probes;
        let probe_id_map = &self.probe_id_map;
        for node in nodes {
            // Find all the inputs
            for (i, input) in node.inputs.iter().enumerate() {
                if let Some(input) = input.get() {
                    input_buffer[i] = buffer[input.0 as usize];
                }
            }

            let output = node.kind.evaluate(
                |probe, val| {
                    let val = (val / dt_per_sample).floor() as usize;
                    let dat = probes.get(probe_id_map.get(&probe)?)?.get_data(val);
                    dat
                },
                &input_buffer, 
                dt_per_sample);
            buffer.push(output);
        }

        for probe in self.probes.values_mut() {
            let input = buffer[probe.probing.0 as usize];
            probe.add_data(input);
        }
    }

    pub fn find_node(&mut self, node_to_find: &Node) -> Option<NodeId> {
        for (i, node) in self.nodes.iter().enumerate() {
            if node == node_to_find {
                return Some(NodeId(i as u32));
            }
        }

        None
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        // Here we check to see if an identical node already exists.
        // This probably won't happen for more complicated nodes,
        // but for things like constants this could indeed happen!
        if let Some(node_id) = self.find_node(&node) {
            node_id
        }else{
            let node_id = NodeId(self.nodes.len() as u32);
            self.nodes.push(node);
            node_id
        }
    }

    pub fn allocate_probe_id(&mut self) -> ProbeId {
        let id = ProbeId(self.probe_id_ctr);
        self.probe_id_ctr += 1;
        id
    }

    pub fn add_probe(&mut self, id: ProbeId, size: usize, probing: NodeId) {
        self.probes.insert(probing, Probe::new(size, probing));
        self.probe_id_map.insert(id, probing);
    }
}

#[derive(Hash,Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeId(u32);

// The point of the probe datastructure is to
// give a simple interface to a probe, and allow
// a fixed size list of data where you can
// effeciently insert some data at the begginging
// and drop the data at the end in one operation
#[derive(Debug)]
pub struct Probe {
    data: Vec<f32>,
    probing: NodeId,

    // The data start is an offset from the beginning
    // of the data vector where the first element is located.
    // The reason for this is so that we can move this pointer
    // instead of moving every element in the data vector,
    // which is a lot more efficient.
    data_start: usize,
}

impl Probe {
    pub fn new(size: usize, probing: NodeId) -> Probe {
        Probe {
            data: vec![0.0; size],
            probing: probing,
            data_start: 0,
        }
    }

    pub fn get_data(&self, loc: usize) -> Option<f32> {
        // The data wraps around. The location has to be less than the lenth of the data,
        // i.e. the size that was given at the start. 
        if loc < self.data.len() {
            // Do this to wrap the pointer around the data vector.
            // All this is again to allow for easy insertion of data
            // at the first element without actually moving
            // any memory around
            let index = (self.data_start + loc) % self.data.len();
            Some(self.data[index])
        }else{
            None
        }
    }

    pub fn add_data(&mut self, data: f32) {
        // A wrapping subtraction
        // has to be done since usize cannot be less than 0,
        // and also because of the purpose of this data
        if self.data_start > 0 {
            self.data_start -= 1;
        }else {
            self.data_start = self.data.len() - 1;
        }

        // Set the first element to the data. This operation
        // also wipes the previous last element clean, two
        // birds with one stone!
        self.data[self.data_start] = data;
    }
}

#[derive(Debug, PartialEq)]
pub struct Node {
    inputs: [MaybeNodeId; MAX_INPUTS],
    kind: NodeType,
}

impl Node {
    pub fn oscillator(freq: NodeId, offset: f32) -> Node {
        let mut inputs = [MaybeNodeId::none(); MAX_INPUTS];
        inputs[0] = freq.maybe();
        Node {
            inputs: inputs,
            kind: NodeType::Oscillator(offset),
        }
    }

    pub fn clamp(thing: NodeId, min: f32, max: f32) -> Node {
        let mut inputs = [MaybeNodeId::none(); MAX_INPUTS];
        inputs[0] = thing.maybe();
        Node {
            inputs: inputs,
            kind: NodeType::Clamp(min, max),
        }
    }

    pub fn square_oscillator(freq: NodeId, offset: f32) -> Node {
        let mut inputs = [MaybeNodeId::none(); MAX_INPUTS];
        inputs[0] = freq.maybe();
        Node {
            inputs: inputs,
            kind: NodeType::SquareOscillator(offset),
        }
    }

    pub fn delay(max: f32, delay: NodeId, probe: ProbeId) -> Node {
        let mut inputs = [MaybeNodeId::none(); MAX_INPUTS];
        inputs[0] = delay.maybe();
        Node {
            inputs: inputs,
            kind: NodeType::Delay(max, probe),
        }
    }

    pub fn constant(constant: f32) -> Node {
        Node {
            inputs: [MaybeNodeId::none(); MAX_INPUTS],
            kind: NodeType::Constant(constant),
        }
    }

    pub fn constant_op(operator: Operator, a: NodeId, b: NodeId) -> Node {
        let mut inputs = [MaybeNodeId::none(); MAX_INPUTS];
        inputs[0] = a.maybe();
        inputs[1] = b.maybe();
        Node {
            inputs: inputs,
            kind: NodeType::ConstantOp(operator),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum NodeType {
    SquareOscillator(f32),
    Oscillator(f32),
    Clamp(f32, f32),
    Constant(f32),
    ConstantOp(Operator),
    Delay(f32, ProbeId), 
}

impl NodeType {
    pub fn is_constant(&self) -> bool {
        use NodeType::*;
        match self {
            SquareOscillator(_) => false,
            Oscillator(_) => false,
            Clamp(_, _) => true,
            Constant(_) => true,
            ConstantOp(_) => true,
            Delay(_, _) => false,
        }
    }

    #[inline]
    pub fn n_inputs(&self) -> usize {
        use NodeType::*;
        match self {
            SquareOscillator(_) => 1,
            Oscillator(_) => 1,
            Clamp(_, _) => 1,
            Constant(_) => 0,
            ConstantOp(_) => 2,
            Delay(_, _) => 1,
        }
    }

    #[inline]
    pub fn evaluate(&mut self, 
                    get_probe_value: impl Fn(ProbeId, f32) -> Option<f32>, 
                    inputs: &[f32; MAX_INPUTS], 
                    dt_per_sample: f32) -> f32 {
        use NodeType::*;
        match self {
            SquareOscillator(t) => {
                *t = (*t + inputs[0].abs() * dt_per_sample) % 1.0;
                ((*t % 1.0) * 2.0).ceil() - 1.0
            },
            Oscillator(t) => {
                *t = (*t + inputs[0].abs() * dt_per_sample) % 1.0;
                (*t * 2.0 * std::f32::consts::PI).sin()
            },
            Clamp(min, max) => {
                inputs[0].max(*min).min(*max)
            },
            Constant(c) => *c,
            ConstantOp(op) => op.evaluate(inputs[0], inputs[1]),
            Delay(max, probe) => {
                let t = inputs[0].max(0.0).min(*max);
                get_probe_value(*probe, t).expect("Expected a valid probe")
            },
        }
    }
}
