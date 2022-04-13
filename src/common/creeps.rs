use crate::common::*;

#[derive(Default)]
pub struct Creeps {
    pub existing: Vec<Creep>,
    pub spawning: Vec<Creep>,
}

impl Creeps {
    pub fn update_existing(&mut self) -> usize {
        let spawning = std::mem::take(&mut self.spawning);
        let (existing, spawning): (Vec<_>, _) = spawning.into_iter().partition(Creep::exists);
        self.spawning = spawning;
        self.existing.retain(Creep::exists);
        self.existing.extend(existing);
        self.spawning.len()
    }
}
