use crate::{
    base::Symbol,
    counter::{CounterExpr, FixedLinearExpr},
};

pub enum Block {
    Symbols(SymbolsBlock),
    RepeatedSymbolsLinear(RepeatedSymbolsLinearBlock),
}

impl Block {
    pub fn size(&self) -> CounterExpr {
        match self {
            Block::Symbols(b) => CounterExpr::Constant(b.size()),
            Block::RepeatedSymbolsLinear(b) => CounterExpr::LinearFixed(b.size()),
        }
    }
}
pub struct SymbolsBlock {
    symbols: Vec<Symbol>,
}

impl SymbolsBlock {
    pub fn new(symbols: &[Symbol]) -> Self {
        Self {
            symbols: symbols.to_vec(),
        }
    }

    pub fn size(&self) -> i64 {
        self.symbols.len() as i64
    }
}

pub struct RepeatedSymbolsLinearBlock {
    block: SymbolsBlock,
    repeat_count: FixedLinearExpr,
}

impl RepeatedSymbolsLinearBlock {
    pub fn new(symbols: &[Symbol], repeat_count: FixedLinearExpr) -> Self {
        Self {
            block: SymbolsBlock::new(symbols),
            repeat_count,
        }
    }

    pub fn size(&self) -> FixedLinearExpr {
        self.repeat_count * self.block.size()
    }
}
