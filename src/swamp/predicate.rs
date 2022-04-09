use dynamic_plan_tree::predicate::*;

#[enum_dispatch(Predicate)]
#[derive(Serialize, Deserialize, FromAny)]
pub enum Predicates {
    True,
    False,
    And(And<Self>),
    Or(Or<Self>),
    Xor(Xor<Self>),
    Not(Not<Self>),
    Nand(Nand<Self>),
    Nor(Nor<Self>),
    Xnor(Xnor<Self>),

    AllSuccess,
    AnySuccess,
    AllFailure,
    AnyFailure,
}
