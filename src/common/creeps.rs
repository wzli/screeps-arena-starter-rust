use crate::common::*;
use dpt::Cast;
use std::collections::HashMap;

pub trait HasCreeps: Sized + 'static {
    fn creeps(&self) -> &Creeps;
    fn creeps_mut(&mut self) -> &mut Creeps;

    fn add_creep(
        plan: Option<&mut dpt::Plan<impl dpt::Config>>,
        creep: Creep,
    ) -> Option<&mut Creep> {
        plan?.cast_mut::<Self>()?.creeps_mut().add_creep(creep)
    }

    fn add_spawning(plan: Option<&mut dpt::Plan<impl dpt::Config>>, creep: Creep) {
        plan.unwrap()
            .cast_mut::<Self>()
            .unwrap()
            .creeps_mut()
            .add_spawning(creep)
    }
}

#[derive(Default)]
pub struct Creeps(pub HashMap<String, Creep>, pub Vec<Creep>);

impl Creeps {
    pub fn add_creep(&mut self, creep: Creep) -> Option<&mut Creep> {
        Some(self.0.entry(get_creep_id(&creep)?).or_insert(creep))
    }
    pub fn add_spawning(&mut self, creep: Creep) {
        self.1.push(creep);
    }
    pub fn check_existence(&mut self) -> usize {
        let spawning = std::mem::take(&mut self.1);
        let (exists, spawning): (Vec<_>, _) = spawning.into_iter().partition(Creep::exists);
        self.1 = spawning;
        self.0.retain(|_, creep| creep.exists());
        for creep in exists {
            self.add_creep(creep);
        }
        self.1.len()
    }
}
