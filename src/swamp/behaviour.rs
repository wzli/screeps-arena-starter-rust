use crate::common::*;
use dynamic_plan_tree::behaviour::*;
use screeps_arena::{
    constants::{prototypes, Part},
    Creep, GameObject, ReturnCode, StructureSpawn,
};

use js_sys::JsString;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

fn get_id(obj: &GameObject) -> Option<JsValue> {
    static mut ID_KEY: Option<JsValue> = None;
    unsafe {
        if let None = ID_KEY {
            ID_KEY = Some(JsValue::from("id"));
        }
        js_sys::Reflect::get(obj, ID_KEY.as_ref().unwrap()).ok()
    }
}

fn get_creep_id(creep: &Creep) -> Option<JsString> {
    get_id(creep)
        .and_then(|x| x.as_f64())
        .map(|x| x.to_string().into())
}

fn get_by_id<T: JsCast>(id: &JsString) -> Option<T> {
    game::utils::get_object_by_id(id).and_then(|x| x.dyn_into::<T>().ok())
}

#[enum_dispatch(Behaviour<C>)]
#[derive(Serialize, Deserialize, FromAny)]
pub enum Behaviours<C: Config> {
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

    RootBehaviour,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RootBehaviour {
    #[serde(skip)]
    spawn: Option<JsString>,
    #[serde(skip)]
    enemy_spawn: Option<JsString>,
    #[serde(skip)]
    creeps: Vec<JsString>,
}

impl<C: Config> Behaviour<C> for RootBehaviour {
    fn status(&self, _plan: &Plan<C>) -> Option<bool> {
        None
    }

    fn on_entry(&mut self, _plan: &mut Plan<C>) {
        let spawns = game::utils::get_objects_by_prototype(prototypes::STRUCTURE_SPAWN);
        self.spawn = spawns
            .iter()
            .find(|x| x.my().unwrap_or(false))
            .map(|x| x.id());
        self.enemy_spawn = spawns
            .iter()
            .find(|x| !x.my().unwrap_or(true))
            .map(|x| x.id());
    }

    fn on_run(&mut self, _plan: &mut Plan<C>) {
        if let Some(id) = &self.spawn {
            let spawn = get_by_id::<StructureSpawn>(id).unwrap();
            let _ = spawn.spawn_creep(&[Part::Move, Part::Attack]);
        }
        let creeps = game::utils::get_objects_by_prototype(prototypes::CREEP);
        let (my_creeps, _enemy_creeps): (Vec<_>, _) = creeps.into_iter().partition(|x| x.my());

        if let Some(id) = &self.enemy_spawn {
            let spawn = get_by_id::<StructureSpawn>(id).unwrap();
            for creep in &my_creeps {
                if let ReturnCode::Ok = creep.attack(&spawn) {
                } else {
                    creep.move_to(&spawn, None);
                }
            }
        }

        self.creeps = my_creeps.iter().filter_map(get_creep_id).collect();
        debug!("{self:?}");
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
