use crate::common::*;
use dpt::behaviour::*;

use std::collections::HashMap;

pub use screeps_arena::{
    constants::{prototypes, Part},
    ResourceType, ReturnCode, StructureContainer, StructureSpawn,
};

trait HasCreeps: Sized + 'static {
    fn creeps(&self) -> &Creeps;
    fn creeps_mut(&mut self) -> &mut Creeps;

    fn add_creep(plan: Option<&mut Plan<impl Config>>, creep: Creep) -> Option<&mut Creep> {
        plan?.cast_mut::<Self>()?.creeps_mut().add_creep(creep)
    }
}

#[derive(Default)]
pub struct Creeps(pub HashMap<String, Creep>);

impl Creeps {
    pub fn add_creep(&mut self, creep: Creep) -> Option<&mut Creep> {
        Some(self.0.entry(get_creep_id(&creep)?).or_insert(creep))
    }
}

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
    AttackBehaviour,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RootBehaviour {
    #[serde(skip)]
    my_spawn: Option<StructureSpawn>,
    #[serde(skip)]
    op_spawn: Option<StructureSpawn>,
    #[serde(skip)]
    my_creeps: Vec<Creep>,
    #[serde(skip)]
    op_creeps: Vec<Creep>,
}

impl<C: Config> Behaviour<C> for RootBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }

    fn on_entry(&mut self, plan: &mut Plan<C>) {
        // get spawns
        let spawns = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_SPAWN);
        let (mut my, mut op): (Vec<_>, _) =
            spawns.into_iter().partition(|x| x.my().unwrap_or(false));
        self.my_spawn = my.pop();
        self.op_spawn = op.pop();

        // create harvest plan
        let mut harvest = HarvestBehaviour::default();
        harvest.my_spawn = self.my_spawn.clone();
        plan.insert(Plan::new(
            C::Behaviour::from_any(harvest).unwrap(),
            "harvest",
            1,
            true,
        ));

        // create attack plan
        let mut attack = AttackBehaviour::default();
        attack.op_spawn = self.op_spawn.clone();
        plan.insert(Plan::new(
            C::Behaviour::from_any(attack).unwrap(),
            "attack",
            1,
            true,
        ));
    }

    fn on_pre_run(&mut self, plan: &mut Plan<C>) {
        // get all creeps
        let creeps = game::utils::get_objects_by_prototype(prototypes::CREEP);
        // partition out my creeps
        let (my_creeps, op_creeps): (Vec<_>, _) = creeps.into_iter().partition(|x| x.my());
        // partition out my creeps with carry
        let (carry_creeps, my_creeps): (Vec<_>, _) = my_creeps
            .into_iter()
            .partition(|x| x.body().iter().find(|x| x.part() == Part::Carry).is_some());
        // partition out my creeps with attack
        let (attack_creeps, my_creeps): (Vec<_>, _) = my_creeps
            .into_iter()
            .partition(|x| x.body().iter().find(|x| x.part() == Part::Attack).is_some());
        // store other creeps
        self.my_creeps = my_creeps;
        self.op_creeps = op_creeps;

        // spawn creeps
        if let Some(spawn) = &self.my_spawn {
            if carry_creeps.len() < 3 {
                let _ = spawn.spawn_creep(&[Part::Move, Part::Carry]);
            } else {
                let _ = spawn.spawn_creep(&[Part::Move, Part::Move, Part::Attack, Part::Attack]);
            }
        }

        // send carry creeps to harvest plan
        for creep in carry_creeps {
            HarvestBehaviour::add_creep(plan.get_mut("harvest"), creep).unwrap();
        }
        // send attack creeps to attack plan
        for creep in attack_creeps {
            AttackBehaviour::add_creep(plan.get_mut("attack"), creep).unwrap();
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct HarvestBehaviour {
    #[serde(skip)]
    my_spawn: Option<StructureSpawn>,
    #[serde(skip)]
    creeps: Creeps,
}

impl HasCreeps for HarvestBehaviour {
    fn creeps(&self) -> &Creeps {
        &self.creeps
    }
    fn creeps_mut(&mut self) -> &mut Creeps {
        &mut self.creeps
    }
}

impl<C: Config> Behaviour<C> for HarvestBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }

    fn on_run(&mut self, _plan: &mut Plan<C>) {
        // find all cointainers with energy
        let containers = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_CONTAINER)
            .iter()
            .filter(|x| x.store().get(ResourceType::Energy).unwrap_or(0) > 20)
            .collect::<Array>();

        for (_id, creep) in &self.creeps.0 {
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

#[derive(Serialize, Deserialize, Default)]
pub struct AttackBehaviour {
    #[serde(skip)]
    op_spawn: Option<StructureSpawn>,
    #[serde(skip)]
    creeps: Creeps,
}

impl HasCreeps for AttackBehaviour {
    fn creeps(&self) -> &Creeps {
        &self.creeps
    }
    fn creeps_mut(&mut self) -> &mut Creeps {
        &mut self.creeps
    }
}

impl<C: Config> Behaviour<C> for AttackBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }

    fn on_run(&mut self, _plan: &mut Plan<C>) {
        // send creeps to attack opponent
        if let Some(spawn) = &self.op_spawn {
            for (_id, creep) in &self.creeps.0 {
                if let ReturnCode::Ok = creep.attack(spawn) {
                } else {
                    creep.move_to(&spawn, None);
                }
            }
        }
        info!("proof");
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
    fn on_pre_run(&mut self, _plan: &mut Plan<C>) {}
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
