use crate::common::*;

use screeps_arena::constants::{prototypes, Part};

use dynamic_plan_tree::{behaviour::*, predicate::Predicates};

#[derive(Serialize, Deserialize)]
pub struct PlanConfig;
impl plan::Config for PlanConfig {
    type Predicate = Predicates;
    type Behaviour = SwampBehaviours<Self>;
}

pub fn enum_dispatch_trait() {
    behaviour::behaviour_trait!();
    predicate::predicate_trait!();
}

/// Default set of built-in behaviours to serve as example template.
#[enum_dispatch(Behaviour<C>)]
#[derive(Serialize, Deserialize, FromAny)]
pub enum SwampBehaviours<C: Config> {
    DefaultBehaviour,

    AllSuccessStatus,
    AnySuccessStatus,
    EvaluateStatus(EvaluateStatus<C>),
    ModifyStatus(ModifyStatus<C>),

    MultiBehaviour(MultiBehaviour<C>),
    RepeatBehaviour(RepeatBehaviour<C>),
    SequenceBehaviour,
    FallbackBehaviour,
    MaxUtilBehaviour,

    EntryBehaviour,
}

#[derive(Serialize, Deserialize)]
pub struct EntryBehaviour;
impl<C: Config> Behaviour<C> for EntryBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }
    fn on_run(&mut self, _plan: &mut Plan<C>) {
        let mut enemy_spawn = None;
        let spawns = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_SPAWN);
        warn!("spawns {}", spawns.len());
        for spawn in spawns {
            if spawn.my().unwrap_or(false) {
                spawn.spawn_creep(&[Part::Move, Part::Attack]);
            } else {
                enemy_spawn = Some(spawn);
            }
        }

        let creeps = game::utils::get_objects_by_prototype(prototypes::CREEP);
        warn!("creeps {}", creeps.len());
        for creep in creeps {
            if creep.my() {
                match &enemy_spawn {
                    Some(t) => {
                        creep.move_to(t.as_ref(), None);
                        creep.attack(t);
                    }
                    None => {}
                }
            }
        }
    }
}
