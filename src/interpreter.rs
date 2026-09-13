use crate::base::{State, Symbol};
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
pub struct ExecutionContext {
    pub state: State,
    pub step: i64,
    pub position: CounterExpr,
}

pub fn apply_transition(
    context: &mut ExecutionContext,
    tape: &mut Tape,
    transition: &Transition,
) -> Option<()> {
    apply_transition_with_debug(context, tape, transition, false)
}

pub fn apply_transition_with_debug(
    context: &mut ExecutionContext,
    tape: &mut Tape,
    transition: &Transition,
    debug: bool,
) -> Option<()> {
    // ステートの確認
    if context.state != transition.from_state() {
        return None;
    }

    // 入力テープの範囲を計算
    let mut io_range = transition.io_range() + context.position;

    // 領域が足りなければ追加
    if io_range.end >= tape.size() {
        let remain = (io_range.end - tape.size()).as_linear_fixed() + 1;
        tape.insert(
            tape.blocks().len() as i64,
            &RepeatedSymbolsLinearBlock::new(
                &SymbolsBlock::new(vec![Symbol::Zero; 1].as_slice()),
                remain,
            )
            .into(),
        );
    }
    if io_range.start < 0.into() {
        let remain = (io_range.start * -1).as_linear_fixed();
        tape.insert(
            0,
            &RepeatedSymbolsLinearBlock::new(
                &SymbolsBlock::new(vec![Symbol::Zero; 1].as_slice()),
                remain,
            )
            .into(),
        );
        io_range = io_range + CounterExpr::from(remain);
        context.position += CounterExpr::from(remain);
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
    assert!(block_rem == 0.into());
    tape.remove_blocks(block_index, cutted_tape.blocks().len() as i64);
    tape.insert_blocks(block_index, transition.output_tape().blocks());

    // 情報の更新
    context.state = transition.to_state();
    context.position += transition.to_position();
    context.step += 1;

    Some(())
}
