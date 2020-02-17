use std::collections::HashMap;
use crate::sound_gen::SampleGiver;

pub struct Variables {
    pub data: Vec<Option<SampleGiver>>,
    name_mapping: HashMap<String, usize>,
    verified: bool,
}

impl Variables {
    pub fn new() -> Variables {
        Variables {
            data: Vec::new(),
            name_mapping: HashMap::new(),
            verified: false,
        }
    }

    pub fn len(&self) -> usize { self.data.len() }

    pub fn add_var(&mut self, name: String, data: SampleGiver) -> usize {
        if let Some(id) = self.name_to_id(&name[..]) {
            self.data[id] = Some(data);
            id
        }else{
            let id = self.len();
            self.data.push(Some(data));
            self.name_mapping.insert(name, id);
            self.verified = false;
            id
        }
    }

    pub fn name_to_id(&self, name: &str) -> Option<usize> {
        self.name_mapping.get(name).map(|v| *v)
    }

    pub fn is_verified(&self) -> bool {
        self.verified
    }

    pub fn verify(&mut self) -> bool {
        // Go through each variable
        // and make sure that it is
        // defined
        for sample_giver in self.data.iter() {
            if sample_giver.is_none() {
                self.verified = false;
                return false;
            }
        }

        self.verified = true;
        true
    }

    pub fn name_to_id_or_add(&mut self, name: &str) -> usize {
        if let Some(id) = self.name_mapping.get(name) {
            *id
        }else {
            let id = self.len();
            self.data.push(None);
            self.name_mapping.insert(name.to_string(), id);
            self.verified = false;
            id
        }
    }
}
