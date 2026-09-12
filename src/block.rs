use crate::{
    base::Symbol,
    counter::{CounterExpr, FixedLinearExpr},
};

#[derive(Clone)]
pub enum Block {
    Symbols(SymbolsBlock),
    RepeatedSymbolsLinear(RepeatedSymbolsLinearBlock),
}

impl Block {
    pub fn reversed(&self) -> Self {
        match self {
            Block::Symbols(b) => Block::Symbols(b.reversed()),
            Block::RepeatedSymbolsLinear(b) => Block::RepeatedSymbolsLinear(b.reversed()),
        }
    }

    pub fn size(&self) -> CounterExpr {
        match self {
            Block::Symbols(b) => CounterExpr::Constant(b.size()),
            Block::RepeatedSymbolsLinear(b) => CounterExpr::LinearFixed(b.size()),
        }
    }

    pub fn as_symbols(&self) -> Option<&SymbolsBlock> {
        match self {
            Block::Symbols(b) => Some(b),
            _ => None,
        }
    }

    pub fn split_at(&self, index: CounterExpr) -> Option<[Vec<Block>; 2]> {
        match self {
            Block::Symbols(b) => {
                let [left, right] = b.split_at(index.as_constant().unwrap())?;
                Some([vec![Block::Symbols(left)], vec![Block::Symbols(right)]])
            }
            Block::RepeatedSymbolsLinear(b) => b.split_at(index.as_linear_fixed().unwrap()),
        }
    }
}

#[derive(Clone)]
pub struct SymbolsBlock {
    symbols: Vec<Symbol>,
}

impl SymbolsBlock {
    pub fn new(symbols: &[Symbol]) -> Self {
        Self {
            symbols: symbols.to_vec(),
        }
    }

    pub fn reversed(&self) -> Self {
        let mut symbols = self.symbols.clone();
        symbols.reverse();
        Self { symbols }
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    pub fn size(&self) -> i64 {
        self.symbols.len() as i64
    }

    pub fn split_at(&self, index: i64) -> Option<[SymbolsBlock; 2]> {
        if index <= 0 || index >= self.size() {
            return None;
        }
        Some([
            SymbolsBlock {
                symbols: self.symbols[..index as usize].to_vec(),
            },
            SymbolsBlock {
                symbols: self.symbols[index as usize..].to_vec(),
            },
        ])
    }
}

#[derive(Clone)]
pub struct RepeatedSymbolsLinearBlock {
    block: SymbolsBlock,
    repeat_count: FixedLinearExpr,
}

impl RepeatedSymbolsLinearBlock {
    pub fn new(symbols: &SymbolsBlock, repeat_count: FixedLinearExpr) -> Self {
        Self {
            block: symbols.clone(),
            repeat_count,
        }
    }

    pub fn reversed(&self) -> Self {
        Self {
            block: self.block.reversed(),
            repeat_count: self.repeat_count,
        }
    }

    pub fn size(&self) -> FixedLinearExpr {
        self.repeat_count * self.block.size()
    }

    pub fn split_at(&self, index: FixedLinearExpr) -> Option<[Vec<Block>; 2]> {
        if index <= FixedLinearExpr::from_i64(0) || index >= self.size() {
            return None;
        }

        // カットするブロック数を計算
        assert!(index.coefficient % self.block.size() == 0);
        let left_cut_block_num = index / self.block.size();
        let rem_in_block = (index % self.block.size()).constant;
        let right_cut_block_num = self.repeat_count - left_cut_block_num - rem_in_block.min(1);

        let mut left_results = vec![];
        let mut right_results = vec![];

        // 左側のブロックを追加
        if left_cut_block_num > FixedLinearExpr::from_i64(0) {
            left_results.push(Block::RepeatedSymbolsLinear(Self {
                block: self.block.clone(),
                repeat_count: left_cut_block_num,
            }));
        }

        // 中間のSymbolsBlockを追加
        if rem_in_block > 0 {
            assert!(rem_in_block < self.block.size());
            let [left, right] = self.block.split_at(rem_in_block).unwrap();
            left_results.push(Block::Symbols(left));
            right_results.push(Block::Symbols(right));
        }

        // 右側のブロックを追加
        if right_cut_block_num > FixedLinearExpr::from_i64(0) {
            right_results.push(Block::RepeatedSymbolsLinear(Self {
                block: self.block.clone(),
                repeat_count: right_cut_block_num,
            }));
        }
        Some([left_results, right_results])
    }
}
