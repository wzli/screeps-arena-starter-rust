use common::*;
use wasm_bindgen::prelude::*;

// this needs to come before behaviour and predicate implementations
pub fn enum_dispatch_trait() {
    use dpt::*;
    behaviour_trait!();
    predicate_trait!();
}

mod logging;
pub mod swamp;

#[cfg(feature = "arena-spawn-and-swamp")]
use swamp as mode;

mod common {
    pub use dpt::{enum_dispatch, Deserialize, FromAny, Serialize};
    pub use dynamic_plan_tree as dpt;
    pub use log::*;
    pub use screeps_arena::{game, prelude::*, Creep, GameObject};

    pub use js_sys::Array;
    pub use js_sys::JsString;
    pub use js_sys::Object;
    pub use wasm_bindgen::JsCast;
    pub use wasm_bindgen::JsValue;

    pub fn get_id(obj: &GameObject) -> Option<JsValue> {
        static mut ID_KEY: Option<JsValue> = None;
        unsafe {
            if let None = ID_KEY {
                ID_KEY = Some(JsValue::from("id"));
            }
            js_sys::Reflect::get(obj, ID_KEY.as_ref().unwrap()).ok()
        }
    }

    pub fn get_creep_id(creep: &Creep) -> Option<JsString> {
        get_id(creep)
            .and_then(|x| x.as_f64())
            .map(|x| x.to_string().into())
    }

    /*
    pub fn get_by_id<T: JsCast>(id: &JsString) -> Option<T> {
        game::utils::get_object_by_id(id).and_then(|x| x.dyn_into::<T>().ok())
    }
    */
}

static mut PLAN: Option<dpt::Plan<mode::Config>> = None;

#[wasm_bindgen(js_name = loop)]
pub fn tick() {
    unsafe {
        match &mut PLAN {
            Some(plan) => plan.run(),
            None => {
                logging::setup_logging(logging::Debug);
                info!("{:?}", game::arena_info());

                PLAN = Some(dpt::Plan::new(
                    mode::behaviour::RootBehaviour::default().into(),
                    "root",
                    1,
                    true,
                ));
            }
        }
    }
}
