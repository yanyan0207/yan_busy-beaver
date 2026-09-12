use crate::{
    base::{Rule, State, Symbol},
    block::{Block, RepeatedSymbolsLinearBlock, SymbolsBlock},
    counter::{CounterExpr, FixedLinearExpr},
    range::Range,
    tape::Tape,
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

    pub fn to_position(&self) -> CounterExpr {
        match self {
            Transition::Rules(t) => CounterExpr::Constant(t.to_position),
            Transition::RepeatedRulesLinear(t) => CounterExpr::LinearFixed(t.to_position),
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
}

pub struct RulesTransition {
    pub rules: Vec<Rule>,
    pub io_range: Range,
    pub to_position: i64,
    pub input_tape: Tape,
    pub output_tape: Tape,
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

        let mut visited = vec![false; (end - start + 1) as usize];
        let mut input_block = vec![Symbol::Zero; (end - start + 1) as usize];
        let mut output_block = vec![Symbol::Zero; (end - start + 1) as usize];
        for (rule, pos) in rules.iter().zip(pos_list.iter()) {
            let pos = (pos - start) as usize;
            if !visited[pos] {
                visited[pos] = true;
                input_block[pos] = rule.read_symbol;
            }
            output_block[pos] = rule.instruction.write_symbol;
        }

        Self {
            io_range,
            rules: rules.to_vec(),
            to_position,
            input_tape: Tape::new(vec![Block::Symbols(SymbolsBlock::new(&input_block))]),
            output_tape: Tape::new(vec![Block::Symbols(SymbolsBlock::new(&output_block))]),
        }
    }

    pub fn reversed(&self) -> Self {
        Self::new(&self.rules.iter().map(|r| r.reversed()).collect::<Vec<_>>())
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
    pub input_tape: Tape,
    pub output_tape: Tape,
}

impl RepeatedRulesLinearTransition {
    pub fn new(rules: &[Rule], repeat_count: FixedLinearExpr) -> Self {
        assert!(repeat_count > FixedLinearExpr::from_i64(0));
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
                end: CounterExpr::LinearFixed((repeat_count - 1) * rule_block.to_position)
                    + rule_io_range.end,
            };
            // リピートブロックサイズ
            let block_size = rule_block.to_position;

            // 重なりの無いリピートの場合
            let (input_tape, output_tape) = if rule_io_range.start == CounterExpr::Constant(0)
                && rule_io_range.end == CounterExpr::Constant(rule_block.to_position) - 1
            {
                let repeat_input_block =
                    RepeatedSymbolsLinearBlock::new(rule_input_block, repeat_count);
                let input_tape = Tape::new(vec![Block::RepeatedSymbolsLinear(repeat_input_block)]);
                let repeat_output_block =
                    RepeatedSymbolsLinearBlock::new(rule_output_block, repeat_count);
                let output_tape =
                    Tape::new(vec![Block::RepeatedSymbolsLinear(repeat_output_block)]);
                (input_tape, output_tape)
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
                let input_blocks = vec![
                    Block::Symbols(rule_input_block.clone()),
                    Block::RepeatedSymbolsLinear(repeat_input_block),
                ];
                let input_tape = Tape::new(input_blocks);

                // 重なりのあるリピートの場合の出力ブロックの作成
                let output_repeat_symbols = &rule_output_symbols[..block_size as usize];
                let output_repeat_block = RepeatedSymbolsLinearBlock::new(
                    &SymbolsBlock::new(output_repeat_symbols),
                    repeat_count - 1,
                );
                let output_blocks = vec![
                    Block::RepeatedSymbolsLinear(output_repeat_block),
                    Block::Symbols(rule_output_block.clone()),
                ];
                let output_tape = Tape::new(output_blocks);
                (input_tape, output_tape)
            };
            Self {
                rule_block,
                repeat_count,
                io_range,
                to_position,
                input_tape,
                output_tape,
            }
        } else {
            let rules = rules.iter().map(|r| r.reversed()).collect::<Vec<_>>();
            let ret = Self::new(&rules, repeat_count);
            ret.reversed()
        }
    }

    pub fn reversed(&self) -> Self {
        Self {
            rule_block: self.rule_block.reversed(),
            repeat_count: self.repeat_count,
            io_range: self.io_range.reversed(),
            to_position: self.to_position * -1,
            input_tape: self.input_tape.reversed(),
            output_tape: self.output_tape.reversed(),
        }
    }
}
