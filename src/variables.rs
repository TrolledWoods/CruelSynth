use std::collections::HashMap;
use crate::sound_gen::SampleGiver;

pub struct Variables {
    data: Vec<(f32, SampleGiver)>,
    name_mapping: HashMap<String, usize>,
}

impl Variables {
    pub fn new() -> Variables {
        Variables {
            data: Vec::new(),
            name_mapping: HashMap::new(),
        }
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, (f32, SampleGiver)> {
        self.data.iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, (f32, SampleGiver)> {
        self.data.iter_mut()
    }

    pub fn add_var(&mut self, name: String, data: SampleGiver) -> usize {
        let id = self.data.len();
        self.data.push((0.0, data));
        self.name_mapping.insert(name, id);
        id
    }

    pub fn name_to_id(&mut self, name: &str) -> Option<usize> {
        self.name_mapping.get(name).map(|v| *v)
    }

    pub fn get_var_sample(&self, id: usize) -> Option<f32> {
        if let Some(val) = self.data.get(id) {
            Some(val.0)
        }else{
            None
        }
    }

    pub fn get_mut_var<'a>(&'a mut self, id: usize) -> Option<&'a mut (f32, SampleGiver)> {
        self.data.get_mut(id)
    }
}
