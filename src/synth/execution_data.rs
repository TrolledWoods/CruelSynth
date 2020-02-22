use std::collections::HashMap;
use super::{ NodeKind, Node, MAX_INPUTS, Synth, Id, MaybeId, Probe };

// TODO: Give this type a nice debug print, that graphs the probes and stuff :)
#[derive(Clone)]
pub struct ExecutionData<'a> {
    node_data: Vec<f32>,

    // The id is the id for a node_data element, as those
    // are the things that can be probed. (It only makes
    // sense to probe the output of a node, right? And the
    // outputs of all nodes are indeed contained in the node_data
    // vector)
    probes: HashMap<Id, ProbeData>,

    sampling_rate: usize,

    synth: &'a Synth,
}

impl ExecutionData<'_> {
    pub fn new<'a>(synth: &'a Synth, sampling_rate: usize) -> ExecutionData<'a> {
        // Set up the probes
        let mut probes = HashMap::with_capacity(synth.probes.len());
        for (&key, probe) in synth.probes.iter() {
            probes.insert(key, ProbeData {
                data: vec![0.0; (probe.max_time * sampling_rate as f32).floor() as usize],
                data_start: 0,
                probing: probe.probing
            });
        }

        println!("{:?}", &probes);

        // Copy the initial data from the synth.
        // This is important as the initial data may contain
        // startup information that changes the function
        // of the synth
        let mut data = synth.initial_data.clone();

        ExecutionData {
            sampling_rate: sampling_rate,
            node_data: data,
            probes: probes,
            synth: synth,
        }
    }

    pub fn get_data(&self, id: Id) -> Option<f32> {
        self.node_data.get(id.as_usize()).map(|v| *v)
    }

    pub fn run(&mut self) {
        let synth = self.synth;
        let data = &mut self.node_data;
        let probes = &mut self.probes;
        let sample_rate = self.sampling_rate as f32; // Convert it to f32 here instead of later to only have to do it once
        let dt_per_sample = 1.0 / self.sampling_rate as f32;

        let mut inputs = [0f32; MAX_INPUTS];
        for node in self.synth.nodes.iter() {
            // Gather all the inputs
            for (i, input) in node.inputs.iter().enumerate() {
                if let Some(id) = input.get() {
                    inputs[i] = data[id.0 as usize];
                }
            }

            // Gather all the data that the node needs
            let (start, end) = node.get_allocated_range();
            let mut range = &mut data[start..end];
            
            // Because the data looks like:
            // [data, data, data, output, output],
            // we want to split this such that this pattern holds.
            let (mut data, mut outputs) = range.split_at_mut(node.kind.n_data_allocations());

            // Because we made sure that all the inputs are
            // of right length(we checked them against the node.kind
            // functions, we assume that the node.kind info
            // was correct and that the other functions did their
            // job, so now this operation should be safe.
            // (it was always memory safe to begin with of course,
            // but I felt like it was such a jank operation it should
            // really be unsafe)
            unsafe {
                node.kind.evaluate(
                    |id, time| {
                        let id = synth.probe_id_map.get(&id)?;
                        probes.get(id).map(|v| v.get_data((time * sample_rate).floor() as usize)).flatten()
                    },
                    data,
                    outputs,
                    &inputs[0..node.kind.n_inputs()],
                    dt_per_sample
                    );
            }
        }

        // Update all the probes
        for (id, probe) in probes.iter_mut() {
            // Get the value that you want to probe
            probe.add_data(data[id.as_usize()]);
        }
    }

    /// This is the method to use if you want to get the left and right channel
    /// or smth. The reason for having it here and not returning them by default
    /// is that you may want more channels from your synth, and it's kind of
    /// inconsistant to get the channels from the run method but get the other
    /// channels from this function, so I combined them.
    pub fn get_node_output(&self, node: &Node, output: usize) -> Option<f32> {
        node.get_output_loc(output).map(|v| self.node_data.get(v).map(|v| *v)).flatten()
    }
}

#[derive(Clone)]
pub struct ProbeData {
    data_start: usize,
    data: Vec<f32>,
    probing: Id
}

impl std::fmt::Debug for ProbeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProbeData ( data_start: {:?}, probing: {:?} )", self.data_start, self.probing)
    }
}

impl ProbeData {
    pub fn new(size: usize, probing: Id) -> ProbeData {
        ProbeData {
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
