pub mod logging;

pub use dpt::{enum_dispatch, Deserialize, FromAny, Serialize};
pub use dynamic_plan_tree as dpt;
pub use js_sys::{Array, JsString};
pub use screeps_arena::{game, prelude::*, Creep, GameObject};
pub use tracing::*;
pub use wasm_bindgen::{prelude::*, JsCast, JsValue};

pub fn get_id(obj: &GameObject) -> Option<JsValue> {
    static mut ID_KEY: Option<JsValue> = None;
    unsafe {
        if ID_KEY.is_none() {
            ID_KEY = Some(JsValue::from("id"));
        }
        js_sys::Reflect::get(obj, ID_KEY.as_ref()?).ok()
    }
}

pub fn get_creep_id(creep: &Creep) -> Option<String> {
    get_id(creep)?.as_f64().map(|x| x.to_string())
}

/*
pub fn get_by_id<T: JsCast>(id: &JsString) -> Option<T> {
    game::utils::get_object_by_id(id).and_then(|x| x.dyn_into::<T>().ok())
}
*/
