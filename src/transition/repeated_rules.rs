use super::RulesTransition;
use crate::transition::Transition;
use crate::{
    base::{Rule, State},
    block::{RepeatedSymbolsLinearBlock, SymbolsBlock},
    counter::{CounterExpr, FixedLinearExpr},
    interpreter::{ExecutionContext, apply_transition},
    range::Range,
    tape::Tape,
};

#[derive(Clone)]
pub struct RepeatedRulesLinearTransition {
    pub rule_block: RulesTransition,
    pub io_range: Range,
    pub to_position: FixedLinearExpr,
    pub repeat_count: FixedLinearExpr,
    pub input_tape: Tape,
    pub output_tape: Tape,
    is_simple_repeat: bool,
}

impl RepeatedRulesLinearTransition {
    pub fn new(rules: &[Rule], repeat_count: FixedLinearExpr) -> Self {
        assert!(repeat_count > 0i64.into());
        let rule_block = RulesTransition::new(rules);

        let rule_io_range = rule_block.io_range;
        let rule_input_block = rule_block.input_tape.blocks()[0].as_symbols().unwrap();
        let rule_input_symbols = rule_input_block.symbols();
        let rule_output_block = rule_block.output_tape.blocks()[0].as_symbols().unwrap();
        let rule_output_symbols = rule_output_block.symbols();
        let to_position = repeat_count * rule_block.to_position;

        assert!(rule_block.to_position != 0);

        if rule_block.to_position > 0 {
            // IO範囲の確定
            let io_range = Range {
                start: rule_io_range.start,
                end: CounterExpr::from((repeat_count - 1) * rule_block.to_position)
                    + rule_io_range.end,
            };
            // リピートブロックサイズ
            let block_size = rule_block.to_position;

            // 重なりの無いリピートの場合
            let (input_tape, output_tape, is_simple_repeat) = if rule_io_range.start == 0.into()
                && rule_io_range.end == CounterExpr::from(rule_block.to_position) - 1
            {
                let repeat_input_block =
                    RepeatedSymbolsLinearBlock::new(rule_input_block, repeat_count);
                let input_tape = Tape::new(vec![repeat_input_block.into()]);
                let repeat_output_block =
                    RepeatedSymbolsLinearBlock::new(rule_output_block, repeat_count);
                let output_tape = Tape::new(vec![repeat_output_block.into()]);
                (input_tape, output_tape, true)
            }
            // 重なりのあるリピートの場合の処理
            else {
                // 重なりのあるリピートの場合の入力ブロックの作成
                let repeat_input_symbols =
                    &rule_input_symbols[rule_input_symbols.len() - block_size as usize..];
                let repeat_input_block = RepeatedSymbolsLinearBlock::new(
                    &SymbolsBlock::new(repeat_input_symbols),
                    repeat_count - 1,
                );
                let input_blocks = vec![rule_input_block.clone().into(), repeat_input_block.into()];
                let input_tape = Tape::new(input_blocks);

                // 重なりのあるリピートの場合の出力ブロックの作成
                let output_repeat_symbols = &rule_output_symbols[..block_size as usize];
                let output_repeat_block = RepeatedSymbolsLinearBlock::new(
                    &SymbolsBlock::new(output_repeat_symbols),
                    repeat_count - 1,
                );
                let output_blocks =
                    vec![output_repeat_block.into(), rule_output_block.clone().into()];
                let output_tape = Tape::new(output_blocks);
                (input_tape, output_tape, false)
            };
            Self {
                rule_block,
                repeat_count,
                io_range,
                to_position,
                input_tape,
                output_tape,
                is_simple_repeat,
            }
        } else {
            let rules = rules.iter().map(|r| r.reversed()).collect::<Vec<_>>();
            let ret = Self::new(&rules, repeat_count);
            ret.reversed()
        }
    }

    pub fn from_state(&self) -> State {
        self.rule_block.from_state()
    }

    pub fn to_state(&self) -> State {
        self.rule_block.to_state()
    }

    pub fn proof_self(&self) -> bool {
        // 本来は3未満でも処理できるが、ここでは3以上であることを前提としている
        assert!(self.repeat_count >= 3i64.into());
        if self.repeat_count < 3i64.into() {
            return true;
        }

        // ステートが同じかどうか確認
        if self.rule_block.from_state() != self.rule_block.to_state() {
            return false;
        }

        // 単純なリピートなら証明不要
        if self.is_simple_repeat {
            return true;
        }

        // 左向きなら逆向きにして証明
        if self.to_position < 0i64.into() {
            let reversed = self.reversed();
            return reversed.proof_self();
        }

        let mut tape = self.input_tape.clone();
        let mut context = ExecutionContext {
            step: 0,
            state: self.from_state(),
            position: self.io_range.start * -1,
        };

        // Ruleトランザクションを2回実行出来て、カウンターが2減ってること
        // Note: 実処理で既に実際二回実行してるのでここでの確認は不要にできる
        let expected_repeat_count = self.repeat_count - 2;
        let t: Transition = self.rule_block.clone().into();
        apply_transition(&mut context, &mut tape, &t);
        apply_transition(&mut context, &mut tape, &t);

        tape.blocks()
            .last()
            .unwrap()
            .as_repeated_symbols_linear()
            .unwrap()
            .repeat_count()
            == expected_repeat_count
    }

    pub fn reversed(&self) -> Self {
        Self {
            rule_block: self.rule_block.reversed(),
            repeat_count: self.repeat_count,
            io_range: self.io_range.reversed(),
            to_position: self.to_position * -1,
            input_tape: self.input_tape.reversed(),
            output_tape: self.output_tape.reversed(),
            is_simple_repeat: self.is_simple_repeat,
        }
    }
}
