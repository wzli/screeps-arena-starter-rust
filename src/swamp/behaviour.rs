use crate::common::*;
use dpt::behaviour::*;

use std::collections::HashMap;

pub use screeps_arena::{
    constants::{prototypes, Part},
    ResourceType, ReturnCode, StructureContainer, StructureSpawn,
};

#[enum_dispatch(Behaviour<C>)]
#[derive(Serialize, Deserialize, FromAny)]
pub enum Behaviours<C: Config> {
    AllSuccessStatus,
    AnySuccessStatus,
    EvaluateStatus(EvaluateStatus<C>),
    ModifyStatus(ModifyStatus<C>),

    MultiBehaviour(MultiBehaviour<C>),
    RepeatBehaviour(RepeatBehaviour<C>),
    SequenceBehaviour,
    FallbackBehaviour,
    MaxUtilBehaviour,

    RootBehaviour,
    HarvestBehaviour,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RootBehaviour {
    #[serde(skip)]
    my_spawn: Option<StructureSpawn>,
    #[serde(skip)]
    op_spawn: Option<StructureSpawn>,
}

impl<C: Config> Behaviour<C> for RootBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }

    fn on_entry(&mut self, plan: &mut Plan<C>) {
        let spawns = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_SPAWN);
        let (mut my, mut op): (Vec<_>, _) =
            spawns.into_iter().partition(|x| x.my().unwrap_or(false));
        self.my_spawn = my.pop();
        self.op_spawn = op.pop();

        plan.insert(Plan::new(
            C::Behaviour::from_any(HarvestBehaviour::new(self.my_spawn.clone())).unwrap(),
            "harvest",
            1,
            true,
        ));
    }

    fn on_run(&mut self, plan: &mut Plan<C>) {
        // get creeps
        let creeps = game::utils::get_objects_by_prototype(prototypes::CREEP);
        let (my_creeps, _op_creeps): (Vec<_>, _) = creeps.into_iter().partition(|x| x.my());

        let (carry_creeps, my_creeps): (Vec<_>, _) = my_creeps
            .into_iter()
            .partition(|x| x.body().iter().find(|x| x.part() == Part::Carry).is_some());

        let (attack_creeps, my_creeps): (Vec<_>, _) = my_creeps
            .into_iter()
            .partition(|x| x.body().iter().find(|x| x.part() == Part::Attack).is_some());
        let _ = my_creeps;

        // spawn creeps
        if let Some(spawn) = &self.my_spawn {
            if carry_creeps.len() < 3 {
                let _ = spawn.spawn_creep(&[Part::Move, Part::Carry]);
            } else {
                let _ = spawn.spawn_creep(&[Part::Move, Part::Move, Part::Attack, Part::Attack]);
            }
        }

        // send creeps to attack opponent
        if let Some(spawn) = &self.op_spawn {
            for creep in attack_creeps {
                if let ReturnCode::Ok = creep.attack(spawn) {
                } else {
                    creep.move_to(&spawn, None);
                }
            }
        }

        // send carry creeps to harvest plan
        for creep in carry_creeps {
            let harvest = plan
                .get_mut("harvest")
                .unwrap()
                .cast_mut::<HarvestBehaviour>()
                .unwrap();
            harvest
                .creeps
                .entry(get_creep_id(&creep).unwrap().into())
                .or_insert(creep);
        }

        //self.creeps = my_creeps.iter().filter_map(get_creep_id).collect();
        // debug!("{self:?}");
    }
}

#[derive(Serialize, Deserialize)]
pub struct HarvestBehaviour {
    #[serde(skip)]
    my_spawn: Option<StructureSpawn>,
    #[serde(skip)]
    creeps: HashMap<String, Creep>,
}

impl HarvestBehaviour {
    fn new(my_spawn: Option<StructureSpawn>) -> Self {
        Self {
            my_spawn,
            creeps: HashMap::new(),
        }
    }
}

impl<C: Config> Behaviour<C> for HarvestBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }

    fn on_run(&mut self, _plan: &mut Plan<C>) {
        let containers = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_CONTAINER)
            .iter()
            .filter(|x| x.store().get(ResourceType::Energy).unwrap_or(0) > 20)
            .collect::<Array>();

        for (_id, creep) in &self.creeps {
            if creep.store().get(ResourceType::Energy).unwrap() == 0 {
                // harvest from containers
                if let Some(closest) = creep.find_closest_by_path(&containers, None) {
                    let closest = JsValue::from(closest).into();
                    if let ReturnCode::Ok = creep.withdraw(&closest, ResourceType::Energy, None) {
                    } else {
                        creep.move_to(&closest, None);
                    }
                }
            } else {
                let spawn = self.my_spawn.as_ref().unwrap();
                if let ReturnCode::Ok = creep.transfer(spawn, ResourceType::Energy, None) {
                } else {
                    creep.move_to(spawn, None);
                }
            }
        }
    }
}

/*
pub trait Behaviour<C: Config>: Send + Sized + 'static {
    fn status(&self, plan: &Plan<C>) -> Option<bool>;
    fn utility(&self, _plan: &Plan<C>) -> f64 {
        0.
    }
    fn on_entry(&mut self, _plan: &mut Plan<C>) {}
    fn on_exit(&mut self, _plan: &mut Plan<C>) {}
    fn on_run(&mut self, _plan: &mut Plan<C>) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
};
*/
