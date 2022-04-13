use common::*;
use wasm_bindgen::prelude::*;

// this needs to come before behaviour and predicate implementations
pub fn enum_dispatch_trait() {
    use dpt::*;
    behaviour_trait!();
    predicate_trait!();
}

pub mod common;
pub mod swamp;

#[cfg(feature = "arena-spawn-and-swamp")]
use swamp as mode;

static mut PLAN: Option<dpt::Plan<mode::Config>> = None;

#[wasm_bindgen(js_name = loop)]
pub fn tick() {
    unsafe {
        match &mut PLAN {
            Some(plan) => plan.run(),
            None => {
                logging::init(logging::Debug);
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
