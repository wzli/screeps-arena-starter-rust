use crate::common::*;
use dpt::behaviour::*;

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
        let spawns = get_objects_by_prototype(prototypes::STRUCTURE_SPAWN);
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
        let mut creeps = Creeps::default();
        creeps.1.push(harvester);
        let harvest = HarvestBehaviour {
            my_spawn: self.my_spawn.clone(),
            creeps,
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
        let creeps = get_objects_by_prototype(prototypes::CREEP);
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
        let containers = get_objects_by_prototype(prototypes::STRUCTURE_CONTAINER)
            .iter()
            .filter(|x| x.store().get(ResourceType::Energy).unwrap_or(0) > 20)
            .collect::<Array>();

        self.creeps.check_existence();
        for creep in self.creeps.0.values() {
            if creep.store().get(ResourceType::Energy).unwrap() == 0 {
                // harvest from containers
                if let Some(closest) = creep.find_closest_by_path(&containers, None).map(obj_from) {
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
