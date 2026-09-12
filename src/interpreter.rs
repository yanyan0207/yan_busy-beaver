use crate::block::Block;
use crate::counter::CounterExpr;
use crate::tape::Tape;
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

pub fn process(context: &mut Context, tape: &mut Tape, transition: &Transition) {
    let io_range = transition.io_range() + context.position;

    let blocks_in_range = tape.split_range(io_range);

    // ブロックが想定通りか比較
    Tape::new(blocks_in_range, tape.position() - io_range.start).compare(transition.input_tape());
}

struct FoundBlockInfo<'a> {
    pub index: i64,
    pub index_in_block: CounterExpr,
    pub block: &'a Block,
}
