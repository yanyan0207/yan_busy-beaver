use crate::{
    base::Symbol,
    counter::{CounterExpr, FixedLinearExpr},
};

pub enum Block {
    Symbols(SymbolsBlock),
    RepeatedSymbolsLinear(RepeatedSymbolsLinearBlock),
}

pub struct SymbolsBlock {
    symbols: Vec<Symbol>,
    current_block_index: Option<i64>,
}

impl SymbolsBlock {
    pub fn new(symbols: Vec<Symbol>) -> Self {
        Self {
            symbols,
            current_block_index: None,
        }
    }
}
pub struct RepeatedSymbolsLinearBlock {
    block: SymbolsBlock,
    repeat_count: FixedLinearExpr,
    current_block_index: Option<FixedLinearExpr>,
}

impl RepeatedSymbolsLinearBlock {
    pub fn new(symbols: Vec<Symbol>, repeat_count: FixedLinearExpr) -> Self {
        Self {
            block: SymbolsBlock::new(symbols),
            repeat_count,
            current_block_index: None,
        }
    }
}
