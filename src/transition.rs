mod repeated_rules;
mod rules;

pub use repeated_rules::RepeatedRulesLinearTransition;
pub use rules::RulesTransition;

use crate::{base::State, counter::CounterExpr, range::Range, tape::Tape};

pub enum Transition {
    Rules(RulesTransition),
    RepeatedRulesLinear(RepeatedRulesLinearTransition),
}

impl From<RulesTransition> for Transition {
    fn from(transition: RulesTransition) -> Self {
        Self::Rules(transition)
    }
}

impl From<RepeatedRulesLinearTransition> for Transition {
    fn from(transition: RepeatedRulesLinearTransition) -> Self {
        Self::RepeatedRulesLinear(transition)
    }
}

impl Transition {
    pub fn io_range(&self) -> Range {
        match self {
            Transition::Rules(t) => t.io_range,
            Transition::RepeatedRulesLinear(t) => t.io_range,
        }
    }

    pub fn to_position(&self) -> CounterExpr {
        match self {
            Transition::Rules(t) => t.to_position.into(),
            Transition::RepeatedRulesLinear(t) => t.to_position.into(),
        }
    }

    pub fn from_state(&self) -> State {
        match self {
            Transition::Rules(t) => t.from_state(),
            Transition::RepeatedRulesLinear(t) => t.from_state(),
        }
    }

    pub fn to_state(&self) -> State {
        match self {
            Transition::Rules(t) => t.to_state(),
            Transition::RepeatedRulesLinear(t) => t.to_state(),
        }
    }

    pub fn input_tape(&self) -> &Tape {
        match self {
            Transition::Rules(t) => &t.input_tape,
            Transition::RepeatedRulesLinear(t) => &t.input_tape,
        }
    }

    pub fn output_tape(&self) -> &Tape {
        match self {
            Transition::Rules(t) => &t.output_tape,
            Transition::RepeatedRulesLinear(t) => &t.output_tape,
        }
    }

    pub fn proof_self(&self) -> bool {
        match self {
            Transition::Rules(_) => true,
            Transition::RepeatedRulesLinear(t) => t.proof_self(),
        }
    }
}
