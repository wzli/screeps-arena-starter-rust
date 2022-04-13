use common::*;

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

#[cfg(any(
    feature = "arena-capture-the-flag",
    feature = "arena-spawn-and-swamp",
    feature = "arena-collect-and-control"
))]
#[wasm_bindgen(js_name = loop)]
pub fn tick() {
    // create plan config
    #[derive(Serialize, Deserialize)]
    pub struct Config;
    impl dpt::Config for Config {
        type Predicate = mode::predicate::Predicates;
        type Behaviour = mode::behaviour::Behaviours<Self>;
    }
    // store static plan
    static mut PLAN: Option<dpt::Plan<mode::Config>> = None;
    // init and run plan
    unsafe {
        match &mut PLAN {
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
            Some(plan) => plan.run(),
        }
    }
}
