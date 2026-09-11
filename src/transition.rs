use crate::{
    base::{Rule, State},
    counter::{CounterExpr, FixedLinearExpr},
    range::Range,
};

pub enum Transition {
    Rules(RulesTransition),
    RepeatedRulesLinear(RepeatedRulesLinearTransition),
}

impl Transition {
    pub fn io_range(&self) -> Range {
        match self {
            Transition::Rules(t) => t.io_range,
            Transition::RepeatedRulesLinear(t) => t.io_range,
        }
    }
}

pub struct RulesTransition {
    pub rules: Vec<Rule>,
    pub io_range: Range,
    pub to_position: i64,
}

impl RulesTransition {
    pub fn new(rules: &[Rule]) -> Self {
        let mut pos = 0;
        let mut pos_list = vec![0];
        pos_list.extend(
            rules
                .iter()
                .map(|r| {
                    pos += r.delta();
                    pos
                })
                .collect::<Vec<i64>>(),
        );
        let to_position = pos_list.pop().unwrap();
        let start = *pos_list.iter().min().unwrap();
        let end = *pos_list.iter().max().unwrap();
        let io_range = Range {
            start: CounterExpr::Constant(start),
            end: CounterExpr::Constant(end),
        };
        Self {
            io_range,
            rules: rules.to_vec(),
            to_position,
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
    pub rule_block: RulesTransition,
    pub io_range: Range,
    pub to_position: FixedLinearExpr,
    pub repeat_count: FixedLinearExpr,
}

impl RepeatedRulesLinearTransition {
    pub fn new(rules: &[Rule], repeat_count: FixedLinearExpr) -> Self {
        let rule_block = RulesTransition::new(rules);

        let mut io_range = rule_block.io_range;
        if rule_block.to_position > 0 {
            io_range.end += CounterExpr::LinearFixed((repeat_count - 1) * rule_block.to_position);
        } else if rule_block.to_position < 0 {
            io_range.start += CounterExpr::LinearFixed((repeat_count - 1) * rule_block.to_position);
        }
        let to_position = repeat_count * rule_block.to_position;

        Self {
            rule_block,
            repeat_count,
            io_range,
            to_position,
        }
    }
}
