use crate::common::*;

pub mod behaviour;
pub mod predicate;

#[derive(Serialize, Deserialize)]
pub struct Config;
impl dpt::Config for Config {
    type Predicate = predicate::Predicates;
    type Behaviour = behaviour::Behaviours<Self>;
}
