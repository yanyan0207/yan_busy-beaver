mod repeated_symbols;
mod symbols;

pub use repeated_symbols::RepeatedSymbolsLinearBlock;
pub use symbols::SymbolsBlock;

use crate::counter::CounterExpr;

#[derive(Clone)]
pub enum Block {
    Symbols(SymbolsBlock),
    RepeatedSymbolsLinear(RepeatedSymbolsLinearBlock),
}

impl From<SymbolsBlock> for Block {
    fn from(block: SymbolsBlock) -> Self {
        Self::Symbols(block)
    }
}

impl From<RepeatedSymbolsLinearBlock> for Block {
    fn from(block: RepeatedSymbolsLinearBlock) -> Self {
        Self::RepeatedSymbolsLinear(block)
    }
}

impl Block {
    pub fn reversed(&self) -> Self {
        match self {
            Block::Symbols(b) => b.reversed().into(),
            Block::RepeatedSymbolsLinear(b) => b.reversed().into(),
        }
    }

    pub fn size(&self) -> CounterExpr {
        match self {
            Block::Symbols(b) => b.size().into(),
            Block::RepeatedSymbolsLinear(b) => b.size().into(),
        }
    }

    pub fn as_symbols(&self) -> Option<&SymbolsBlock> {
        match self {
            Block::Symbols(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_repeated_symbols_linear(&self) -> Option<&RepeatedSymbolsLinearBlock> {
        match self {
            Block::RepeatedSymbolsLinear(b) => Some(b),
            _ => None,
        }
    }

    pub fn split_at(&self, index: CounterExpr) -> Option<[Vec<Block>; 2]> {
        match self {
            Block::Symbols(b) => {
                let [left, right] = b.split_at(index.as_constant().unwrap())?;
                Some([vec![left.into()], vec![right.into()]])
            }
            Block::RepeatedSymbolsLinear(b) => b.split_at(index.as_linear_fixed()),
        }
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Block::Symbols(a), Block::Symbols(b)) => a == b,
            (Block::Symbols(a), Block::RepeatedSymbolsLinear(b)) => {
                RepeatedSymbolsLinearBlock::new(a, 1i64.into()) == *b
            }
            (Block::RepeatedSymbolsLinear(a), Block::Symbols(b)) => {
                RepeatedSymbolsLinearBlock::new(b, 1i64.into()) == *a
            }
            (Block::RepeatedSymbolsLinear(a), Block::RepeatedSymbolsLinear(b)) => a == b,
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Symbols(block) => write!(f, "[{block}]"),
            Block::RepeatedSymbolsLinear(block) => {
                write!(f, "[{}]({})", block.block, block.repeat_count)
            }
        }
    }
}
