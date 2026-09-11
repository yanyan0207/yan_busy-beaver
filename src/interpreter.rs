use crate::block::Block;
use crate::counter::CounterExpr;
use crate::transition::{self, RepeatedRulesLinearTransition, RulesTransition, Transition};
use crate::{base::State, counter::FixedLinearExpr};

pub struct BlockInfo {
    pub start: CounterExpr,
    pub end: CounterExpr,
    pub size: CounterExpr,
    pub index: i64,
    pub is_current: bool,
    pub current_index_in_block: Option<CounterExpr>,
}
pub struct Context {
    pub step: i64,
    pub state: State,
    pub position: CounterExpr,
    pub current_block_index: i64,
    pub blocks: Vec<Block>,
    pub block_infos: Vec<BlockInfo>,
}

fn process_rules_transition(
    context: &mut Context,
    blocks: &mut [Block],
    transition: &RulesTransition,
) {
}

fn process_repeated_rules_linear_transition(
    context: &mut Context,
    blocks: &mut [Block],
    transition: &RepeatedRulesLinearTransition,
) {
}

pub fn process(context: &mut Context, blocks: &mut [Block], transition: &Transition) {
    match transition {
        Transition::Rules(rules_transition) => {
            process_rules_transition(context, blocks, rules_transition);
        }
        Transition::RepeatedRulesLinear(repeated_rules_linear_transition) => {
            process_repeated_rules_linear_transition(
                context,
                blocks,
                repeated_rules_linear_transition,
            );
        }
    }
}
