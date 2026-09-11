use crate::{
    base::{Rule, State},
    counter::FixedLinearExpr,
};

pub enum Transition {
    Rules(RulesTransition),
    RepeatedRulesLinear(RepeatedRulesLinearTransition),
}

pub struct RulesTransition {
    rules: Vec<Rule>,
}

impl RulesTransition {
    pub fn new(rules: &[Rule]) -> Self {
        Self {
            rules: rules.to_vec(),
        }
    }
    pub fn from_state(&self) -> State {
        self.rules[0].current_state
    }
    pub fn to_state(&self) -> State {
        self.rules[self.rules.len() - 1].instruction.next_state
    }
}

pub struct RepeatedRulesLinearTransition {
    rule_block: RulesTransition,
    repeat_count: FixedLinearExpr,
}

impl RepeatedRulesLinearTransition {
    pub fn new(rules: &[Rule], repeat_count: FixedLinearExpr) -> Self {
        Self {
            rule_block: RulesTransition::new(rules),
            repeat_count,
        }
    }
}
