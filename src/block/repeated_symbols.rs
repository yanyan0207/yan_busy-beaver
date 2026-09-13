use super::{Block, SymbolsBlock};
use crate::counter::FixedLinearExpr;

#[derive(Clone, Eq)]
pub struct RepeatedSymbolsLinearBlock {
    pub(crate) block: SymbolsBlock,
    pub(crate) repeat_count: FixedLinearExpr,
}

impl RepeatedSymbolsLinearBlock {
    pub fn new(symbols: &SymbolsBlock, repeat_count: FixedLinearExpr) -> Self {
        Self {
            block: symbols.clone(),
            repeat_count,
        }
    }

    pub fn repeat_count(&self) -> FixedLinearExpr {
        self.repeat_count
    }

    pub fn set_repeat_count(&mut self, repeat_count: FixedLinearExpr) {
        self.repeat_count = repeat_count;
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
        if index <= 0i64.into() || index >= self.size() {
            return None;
        }

        // カットするブロック数を計算
        assert!(index.coefficient % self.block.size() == 0);
        let left_cut_block_num = index / self.block.size();
        let rem_in_block = (index % self.block.size()).constant;
        let right_cut_block_num = self.repeat_count - left_cut_block_num - rem_in_block.min(1);

        let mut left_results = vec![];
        let mut right_results = vec![];

        // 左のリピートブロック
        if left_cut_block_num > 0i64.into() {
            let mut left_block = self.clone();
            left_block.set_repeat_count(left_cut_block_num);
            left_block.block.symbols.rotate_left(rem_in_block as usize);
            left_results = vec![left_block.into()];
        }

        // 右のリピートブロック
        if right_cut_block_num > 0i64.into() {
            let mut right_block = self.clone();
            right_block.set_repeat_count(right_cut_block_num);
            right_block
                .block
                .symbols
                .rotate_right((self.block.size() - rem_in_block) as usize);
            right_results = vec![right_block.into()];
        }

        // 外側のSymbolsBlockを追加
        if rem_in_block > 0 {
            assert!(rem_in_block < self.block.size());
            let [left, right] = self.block.split_at(rem_in_block).unwrap();
            left_results.insert(0, left.into());
            right_results.push(right.into());
        }

        Some([left_results, right_results])
    }
}

impl PartialEq for RepeatedSymbolsLinearBlock {
    fn eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        // リピートブロックの比較
        self.block.symbols.repeat(other.block.size() as usize)
            == other.block.symbols.repeat(self.block.size() as usize)
    }
}
