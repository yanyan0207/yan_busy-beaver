use crate::{
    base::{Rule, State, Symbol},
    block::SymbolsBlock,
    range::Range,
    tape::Tape,
};

#[derive(Clone)]
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
            start: start.into(),
            end: end.into(),
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
            input_tape: Tape::new(vec![SymbolsBlock::new(&input_block).into()]),
            output_tape: Tape::new(vec![SymbolsBlock::new(&output_block).into()]),
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
