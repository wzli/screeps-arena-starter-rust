use common::*;
use wasm_bindgen::prelude::*;

// this needs to come before behaviour and predicate implementations
pub fn enum_dispatch_trait() {
    behaviour::behaviour_trait!();
    predicate::predicate_trait!();
}

mod logging;
pub mod swamp;

//#[cfg(feature = "arena-spawn-and-swamp")]
use swamp as mode;

mod common {
    pub use dynamic_plan_tree::*;
    pub use log::*;
    pub use screeps_arena::{game, prelude::*};
}

static mut PLAN: Option<Plan<mode::PlanConfig>> = None;

#[wasm_bindgen(js_name = loop)]
pub fn tick() {
    unsafe {
        match &mut PLAN {
            Some(plan) => plan.run(),
            None => {
                logging::setup_logging(logging::Debug);
                info!("{:?}", game::arena_info());

                PLAN = Some(Plan::new(
                    mode::behaviour::RootBehaviour::default().into(),
                    "root",
                    1,
                    true,
                ));
            }
        }
    }
}
