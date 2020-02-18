/// The contents of this NodeId type cannot
/// ever be equal to "NIL_NODE_ID", because
/// that is essentially null
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
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
    nodes: Vec<Node>
}

impl Synth {
    pub fn new() -> Synth {
        Synth {
            nodes: Vec::new(),
        }
    }

    pub fn run(&mut self, buffer: &mut Vec<f32>, dt_per_sample: f32) {
        buffer.clear();

        let mut input_buffer = [0f32; MAX_INPUTS];
        for node in self.nodes.iter_mut() {
            // Find all the inputs
            for (i, input) in node.inputs.iter().enumerate() {
                if let Some(input) = input.get() {
                    input_buffer[i] = buffer[input.0 as usize];
                }
            }

            let output = node.kind.evaluate(&input_buffer, dt_per_sample);
            buffer.push(output);
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

    pub fn constant(constant: f32) -> Node {
        Node {
            inputs: [MaybeNodeId::none(); MAX_INPUTS],
            kind: NodeType::Constant(constant),
        }
    }

    pub fn constant_op(constant_op: ConstantOp, a: NodeId, b: NodeId) -> Node {
        let mut inputs = [MaybeNodeId::none(); MAX_INPUTS];
        inputs[0] = a.maybe();
        inputs[1] = b.maybe();
        Node {
            inputs: inputs,
            kind: NodeType::ConstantOp(constant_op),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum NodeType {
    Oscillator(f32),
    Constant(f32),
    ConstantOp(ConstantOp),
}

impl NodeType {
    pub fn is_constant(&self) -> bool {
        use NodeType::*;
        match self {
            Oscillator(_) => false,
            Constant(_) => true,
            ConstantOp(_) => true,
        }
    }

    #[inline]
    pub fn n_inputs(&self) -> usize {
        use NodeType::*;
        match self {
            Oscillator(_) => 1,
            Constant(_) => 0,
            ConstantOp(_) => 2,
        }
    }

    #[inline]
    pub fn evaluate(&mut self, inputs: &[f32; MAX_INPUTS], dt_per_sample: f32) -> f32 {
        use NodeType::*;
        match self {
            Oscillator(t) => {
                *t += inputs[0] * dt_per_sample;
                (*t * 2.0 * std::f32::consts::PI).sin()
            },
            Constant(c) => *c,
            ConstantOp(op) => op.run(inputs[0], inputs[1]),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ConstantOp {
    Add,
    Sub,
    Mult,
    Div,
    Mod,
}

impl ConstantOp {
    #[inline]
    pub fn run(&self, a: f32, b: f32) -> f32 {
        use ConstantOp::*;
        match self {
            Add => a + b,
            Sub => a - b,
            Mult => a * b,
            Div => a / b,
            Mod => a % b,
        }
    }
}
