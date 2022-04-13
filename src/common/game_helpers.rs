use crate::common::*;
use wasm_bindgen::{JsCast, JsValue};

/// use js_sys::Reflect instead of wasm_bindgen to get id attribute
pub fn get_id(obj: &GameObject) -> Option<JsValue> {
    static mut ID_KEY: Option<JsValue> = None;
    unsafe {
        if ID_KEY.is_none() {
            ID_KEY = Some(JsValue::from("id"));
        }
        js_sys::Reflect::get(obj, ID_KEY.as_ref()?).ok()
    }
}

/// this function is required a but causes creep.id() to panic
pub fn get_creep_id(creep: &Creep) -> Option<String> {
    get_id(creep)?.as_f64().map(|x| x.to_string())
}

/// helper function to get by id and cast
pub fn get_by_id<T: JsCast>(id: &JsString) -> Option<T> {
    game::utils::get_object_by_id(id).and_then(|x| x.dyn_into::<T>().ok())
}

/// convert js_sys::Object to GameObject
pub fn obj_from(obj: js_sys::Object) -> GameObject {
    JsValue::from(obj).into()
}

pub fn ok_or_move_to(err: ReturnCode, creep: &Creep, target: &GameObject) {
    match err {
        ReturnCode::Ok => {}
        ReturnCode::NotInRange => match creep.move_to(target, None) {
            ReturnCode::Ok => {}
            err => warn!(Error=?err, "Move"),
        },
        err => warn!(Error=?err, "Unexpected"),
    }
}

pub fn gen_parts(parts_list: &[(Part, usize)]) -> Vec<Part> {
    parts_list
        .iter()
        .flat_map(|(part, n)| vec![*part; *n])
        .collect()
}
