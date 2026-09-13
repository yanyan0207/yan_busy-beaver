use crate::base::Symbol;
use crate::block::Block;
use crate::block::{RepeatedSymbolsLinearBlock, SymbolsBlock};
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
    process_with_debug(context, tape, transition, false)
}

pub fn process_with_debug(
    context: &mut Context,
    tape: &mut Tape,
    transition: &Transition,
    debug: bool,
) -> Option<()> {
    // 入力テープの範囲を計算
    let mut io_range = transition.io_range() + context.position;

    // 領域が足りなければ追加
    if io_range.end >= tape.size() {
        let remain = (io_range.end - tape.size()).as_linear_fixed() + 1;
        tape.insert(
            tape.blocks().len() as i64,
            &Block::RepeatedSymbolsLinear(RepeatedSymbolsLinearBlock::new(
                &SymbolsBlock::new(vec![Symbol::Zero; 1].as_slice()),
                remain,
            )),
        );
    }
    if io_range.start < CounterExpr::Constant(0) {
        let remain = (io_range.start * -1).as_linear_fixed();
        tape.insert(
            0,
            &Block::RepeatedSymbolsLinear(RepeatedSymbolsLinearBlock::new(
                &SymbolsBlock::new(vec![Symbol::Zero; 1].as_slice()),
                remain,
            )),
        );
        io_range = io_range + CounterExpr::LinearFixed(remain);
        context.position += CounterExpr::LinearFixed(remain);
        crate::debug_println!(
            debug,
            "  io_range {}..={} (after left extension)",
            io_range.start,
            io_range.end
        );
    }

    // テープから
    let blocks_in_range = tape.split_range(io_range);

    // ブロックが想定通りか比較
    let cutted_tape = Tape::new(blocks_in_range);
    let is_equal = Tape::compare_with_debug(&cutted_tape, transition.input_tape(), debug);
    crate::debug_println!(
        debug,
        "  input compare: {}",
        if is_equal { "MATCH" } else { "MISMATCH" }
    );
    if !is_equal {
        crate::debug_println!(
            debug,
            "  input mismatch at {}..={}",
            io_range.start,
            io_range.end
        );
        crate::debug_println!(debug, "    actual   {}", cutted_tape);
        crate::debug_println!(debug, "    expected {}", transition.input_tape());
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
