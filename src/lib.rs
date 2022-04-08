use common::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

mod logging;
pub mod swamp;

//#[cfg(feature = "arena-spawn-and-swamp")]
use swamp as mode;

mod common {
    pub use dynamic_plan_tree::*;
    pub use log::*;
    pub use screeps_arena::{game, prelude::*};
}

static PLAN: Lazy<Mutex<Plan<mode::PlanConfig>>> =
    Lazy::new(|| Mutex::new(Plan::new(mode::EntryBehaviour.into(), "root", 1, true)));

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn tick() {
    let tick = game::utils::get_ticks();

    if tick == 1 {
        logging::setup_logging(logging::Info);
        let info = game::arena_info();
        warn!("arena_info: {:?}", info);
    }

    PLAN.lock().unwrap().run();
}
