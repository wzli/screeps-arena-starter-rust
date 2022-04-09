use crate::common::*;

pub mod behaviour;
pub mod predicate;

#[derive(Serialize, Deserialize)]
pub struct PlanConfig;
impl plan::Config for PlanConfig {
    type Predicate = predicate::Predicates;
    type Behaviour = behaviour::Behaviours<Self>;
}
