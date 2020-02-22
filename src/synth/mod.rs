use crate::operator::Operator;
use std::collections::HashMap;
use std::fmt;

mod execution_data;
pub use execution_data::ExecutionData;

/// The contents of this Id type cannot
/// ever be equal to "NIL_NODE_ID", because
/// that is essentially null
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Id(pub u32);

impl Id {
    #[inline]
    pub fn maybe(self) -> MaybeId {
        MaybeId(self.0)
    }

    #[inline]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

const NIL_NODE_ID: u32 = std::u32::MAX;

/// Just an internal representation to save
/// space instead of using an option(and not introduce
/// a null value on Ids)
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct MaybeId(u32);

impl MaybeId {
    #[inline]
    fn none() -> MaybeId {
        MaybeId(NIL_NODE_ID)
    }

    #[inline]
    fn from_option(option: Option<Id>) -> MaybeId {
        match option {
            Some(v) => MaybeId(v.0),
            None => MaybeId::none(),
        }
    }

    #[inline]
    fn get(self) -> Option<Id> {
        if self.0 == NIL_NODE_ID {
            None
        }else{
            Some(Id(self.0))
        }
    }

    #[inline]
    fn get_usize(self) -> Option<usize> {
        if self.0 == NIL_NODE_ID {
            None
        }else{
            Some(self.0 as usize)
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
pub struct Synth {
    // The nodes in the synth
    nodes: Vec<Node>,

    // The allocations of data from the nodes
    data_allocations: Vec<MaybeId>,
    initial_data: Vec<f32>,

    // Probes are stored in the ExecutionData struct,
    // so this is just a place where that probe
    // is referenced.
    probes: HashMap<Id, Probe>,
    probe_id_map: HashMap<Id, Id>,
    probe_id_ctr: u32,
}

impl fmt::Debug for Synth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Synth:")?;
        writeln!(f, "  Nodes:")?;
        for (i, node) in self.nodes.iter().enumerate() {
            writeln!(f, "    {}: {:?}", i, node)?;
        }

        writeln!(f, "  Probes:")?;
        for probe in self.probes.iter() {
            writeln!(f, "    {:?}", probe);
        }

        Ok(())
    }
}

impl Synth {
    pub fn new() -> Synth {
        Synth {
            nodes: Vec::new(),
            data_allocations: Vec::new(),
            initial_data: Vec::new(),

            probes: HashMap::new(),
            probe_id_map: HashMap::new(),
            probe_id_ctr: 0,
        }
    }

    pub fn get_node_data<'a>(&'a self, node: Id) -> Option<&'a [f32]> {
        if let Some(node) = self.nodes.get(node.0 as usize) {
            Some(&self.initial_data[node.data_loc.0 as usize..node.data_loc.0 as usize + node.kind.n_data_allocations()])
        }else{
            None
        }
    }

    pub fn get_node<'a>(&'a self, node: Id) -> Option<&'a Node> {
        self.nodes.get(node.as_usize())
    }

    pub fn get_node_output(&self, node: Id) -> Option<Id> {
        self.nodes.get(node.as_usize()).map(|node| Id(node.get_output_loc(0).unwrap() as u32))
    }

    pub fn add_node(&mut self, kind: NodeKind, inputs: &[Id], data: &[f32]) -> Id {
        let node_id = Id(self.nodes.len() as u32);
        let alloc_loc = Id(self.initial_data.len() as u32);
        data.iter().for_each(|&v| self.initial_data.push(v));
        data.iter().for_each(|_| self.data_allocations.push(node_id.maybe()));
        // Add the output nodes(only 1 possible so far
        for _ in 0..1 {
            self.initial_data.push(0.0);
            self.data_allocations.push(node_id.maybe());
        }
        let mut input_arr = [MaybeId::none(); MAX_INPUTS];
        inputs.iter().enumerate().for_each(|(i, &v)| input_arr[i] = v.maybe());
        self.nodes.push(Node { inputs: input_arr, data_loc: alloc_loc, kind: kind });
        node_id
    }

    pub fn allocate_probe_id(&mut self) -> Id {
        let id = Id(self.probe_id_ctr);
        self.probe_id_ctr += 1;
        id
    }

    pub fn add_probe(&mut self, id: Id, time: f32, probing: Id) {
        self.probes.insert(probing, Probe { max_time: time, probing: probing });
        self.probe_id_map.insert(id, probing);
    }
}

// A basic definition of a probe
#[derive(Debug)]
pub struct Probe {
    pub probing: Id,
    pub max_time: f32,
}

#[derive(Debug, PartialEq)]
pub struct Node {
    inputs: [MaybeId; MAX_INPUTS],
    data_loc: Id,
    kind: NodeKind,
}

impl Node {
    // Nodes are allocated like this: [data, data, data, output, output].
    // Most nodes of course only have 1 output and 0 - 1 piece of data,
    // but this varies from node to node.
    pub fn get_allocated_range(&self) -> (usize, usize) {
        let size = self.kind.n_data_allocations() + 1; // Right now only 1 output is supported
        (self.data_loc.0 as usize, self.data_loc.0 as usize + size)
    }

    pub fn get_output_loc(&self, output: usize) -> Option<usize> {
        if output < self.kind.n_outputs() {
            Some(self.data_loc.as_usize() + self.kind.n_data_allocations())
        }else{
            None
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum NodeKind {
    SquareOscillator,
    Oscillator,
    Linear(f32),
    Sequence(Vec<f32>),
    Clamp(f32, f32),
    Constant(f32),
    ConstantOp(Operator),
    Delay(f32, Id), 
}

impl NodeKind {
    pub fn is_constant(&self) -> bool {
        use NodeKind::*;
        match self {
            SquareOscillator => false,
            Oscillator => false,
            Linear(_) => false,
            Sequence(_) => true,
            Clamp(_, _) => true,
            Constant(_) => true,
            ConstantOp(_) => true,
            Delay(_, _) => false,
        }
    }

    #[inline]
    pub fn n_inputs(&self) -> usize {
        use NodeKind::*;
        match self {
            SquareOscillator => 1,
            Oscillator => 1,
            Linear(_) => 1,
            Sequence(_) => 1,
            Clamp(_, _) => 1,
            Constant(_) => 0,
            ConstantOp(_) => 2,
            Delay(_, _) => 1,
        }
    }

    #[inline]
    pub fn n_outputs(&self) -> usize {
        // TODO: Actually make this a proper function
        // to support more numbers of outputs
        1
    }

    #[inline]
    pub fn n_data_allocations(&self) -> usize {
        use NodeKind::*;
        match self {
            SquareOscillator => 1,
            Oscillator => 1,
            Linear(_) => 1,
            Sequence(_) => 0,
            Clamp(_, _) => 0,
            Constant(_) => 0,
            ConstantOp(_) => 0,
            Delay(_, _) => 0,
        }
    }

    /// We here assume that the inputs and data are of the
    /// correct length, i.e. the values of the "n_inputs" and
    /// "n_data_allocations" functions
    #[inline]
    pub unsafe fn evaluate(&self, 
                    get_probe_value: impl Fn(Id, f32) -> Option<f32>, 
                    data: &mut [f32],
                    outputs: &mut [f32],
                    inputs: &[f32], 
                    dt_per_sample: f32) {
        use NodeKind::*;
        match self {
            SquareOscillator => {
                data[0] = (data[0] + inputs[0].abs() * dt_per_sample) % 1.0;
                outputs[0] = ((data[0] % 1.0) * 2.0).ceil() - 1.0;
            },
            Oscillator => {
                data[0] = (data[0] + inputs[0].abs() * dt_per_sample) % 1.0;
                outputs[0] = (data[0] * 2.0 * std::f32::consts::PI).sin();
            },
            Linear(max) => {
                let value = (data[0] + inputs[0] * dt_per_sample) % max;
                data[0] = value;
                outputs[0] = value;
            },
            Sequence(sequence) => {
                let input = inputs[0].floor();
                let length = sequence.len() as f32;
                // Clamp it in a looping fashion
                let loc = (input - ((input / length).floor() * length).floor()) as usize;
                outputs[0] = sequence[loc];
            },
            Clamp(min, max) => {
                outputs[0] = inputs[0].max(*min).min(*max);
            },
            Constant(c) => outputs[0] = *c,
            ConstantOp(op) => outputs[0] = op.evaluate(inputs[0], inputs[1]),
            Delay(max, probe) => {
                let t = inputs[0].max(0.001).min(*max);
                outputs[0] = get_probe_value(*probe, t).expect("Expected a valid probe");
            },
        }
    }
}
