use crate::counter::CounterExpr;
use crate::tape::Tape;
use crate::transition::Transition;

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
    pub position: CounterExpr,
}

pub fn process(context: &mut Context, tape: &mut Tape, transition: &Transition) -> Option<()> {
    let io_range = transition.io_range() + context.position;

    let blocks_in_range = tape.split_range(io_range);

    // ブロックが想定通りか比較
    let cutted_tape = Tape::new(blocks_in_range);
    let is_equal = Tape::compare(&cutted_tape, transition.input_tape());
    if !is_equal {
        return None;
    }

    // 想定通りなら入れ替え
    let (block_index, block_rem) = tape.find_block(io_range.start);
    assert!(block_rem == CounterExpr::Constant(0));
    tape.remove_blocks(block_index, cutted_tape.blocks().len() as i64);
    tape.insert_blocks(block_index, transition.output_tape().blocks());

    // 情報の更新
    context.position += transition.to_position();
    context.step += 1;

    Some(())
}
