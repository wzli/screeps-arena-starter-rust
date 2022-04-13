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

    fn add_spawning(plan: Option<&mut Plan<impl Config>>, creep: Creep) {
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

    attack_parts: Vec<Part>,
}

impl<C: Config> Behaviour<C> for RootBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }

    fn on_entry(&mut self, plan: &mut Plan<C>) {
        // set params
        self.attack_parts = gen_parts(&[(Part::Move, 5), (Part::Attack, 5)]);

        // get spawns
        let spawns = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_SPAWN);
        let (mut my, mut op): (Vec<_>, _) =
            spawns.into_iter().partition(|x| x.my().unwrap_or(false));
        self.my_spawn = my.pop();
        self.op_spawn = op.pop();

        // spawn harvester
        let harvester = self
            .my_spawn
            .as_ref()
            .unwrap()
            .spawn_creep(&gen_parts(&[(Part::Carry, 5), (Part::Move, 2)]))
            .unwrap();

        // create harvest plan
        let harvest = HarvestBehaviour {
            my_spawn: self.my_spawn.clone(),
            creeps: Creeps(HashMap::new(), vec![harvester]),
        };
        plan.insert(Plan::new(
            C::Behaviour::from_any(harvest).unwrap(),
            "harvest",
            1,
            true,
        ));

        // create attack plan
        let attack = AttackBehaviour {
            op_spawn: self.op_spawn.clone(),
            ..AttackBehaviour::default()
        };
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
        self.my_creeps = my_creeps;
        self.op_creeps = op_creeps;

        // spawn attack creeps
        if let Ok(creep) = self
            .my_spawn
            .as_ref()
            .unwrap()
            .spawn_creep(&self.attack_parts)
        {
            AttackBehaviour::add_spawning(plan.get_mut("attack"), creep);
        }
    }
}

#[derive(Serialize, Deserialize)]
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
        let spawn = self.my_spawn.as_ref().unwrap();

        // find all cointainers with energy
        let containers = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_CONTAINER)
            .iter()
            .filter(|x| x.store().get(ResourceType::Energy).unwrap_or(0) > 20)
            .collect::<Array>();

        self.creeps.check_existence();
        for creep in self.creeps.0.values() {
            if creep.store().get(ResourceType::Energy).unwrap() == 0 {
                // harvest from containers
                if let Some(closest) = creep.find_closest_by_path(&containers, None) {
                    let closest = JsValue::from(closest).into();
                    ok_or_move_to(
                        creep.withdraw(&closest, ResourceType::Energy, None),
                        creep,
                        &closest,
                    );
                }
            } else {
                ok_or_move_to(
                    creep.transfer(spawn, ResourceType::Energy, None),
                    creep,
                    spawn,
                );
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
        let spawn = self.op_spawn.as_ref().unwrap();
        // send creeps to attack opponent
        self.creeps.check_existence();
        for creep in self.creeps.0.values() {
            ok_or_move_to(creep.attack(spawn), creep, spawn);
        }
    }
}

pub fn gen_parts(parts_list: &[(Part, usize)]) -> Vec<Part> {
    parts_list
        .iter()
        .flat_map(|(part, n)| vec![*part; *n])
        .collect()
}

fn ok_or_move_to(err: ReturnCode, creep: &Creep, target: &GameObject) {
    match err {
        ReturnCode::Ok => {}
        ReturnCode::NotInRange => match creep.move_to(target, None) {
            ReturnCode::Ok => {}
            err => warn!(" error {err:?}"),
        },
        err => warn!("unexpected error {err:?}"),
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
