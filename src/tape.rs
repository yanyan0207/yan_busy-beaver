use crate::block::Block;
use crate::counter::CounterExpr;
use crate::range::Range;

pub struct Tape {
    blocks: Vec<Block>,
    position: CounterExpr,
}

impl Tape {
    pub fn new(blocks: Vec<Block>, position: CounterExpr) -> Self {
        Self { blocks, position }
    }

    pub fn position(&self) -> CounterExpr {
        self.position
    }

    pub fn size(&self) -> CounterExpr {
        self.blocks
            .iter()
            .fold(CounterExpr::Constant(0), |acc, block| acc + block.size())
    }

    pub fn block(&self, block_index: i64) -> &Block {
        &self.blocks[block_index as usize]
    }

    pub fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }

    pub fn compare(&self, other: &Tape) -> bool {
        // 比較対象のテープと現在のテープのブロック構造を比較する
        false
    }

    pub fn reversed(&self) -> Self {
        Self {
            blocks: self.blocks.iter().rev().map(|b| b.reversed()).collect(),
            position: self.size() - self.position - 1,
        }
    }

    pub fn insert(&mut self, block_index: i64, block: &Block) {
        self.blocks.insert(block_index as usize, block.clone());
    }

    pub fn insert_blocks(&mut self, block_index: i64, blocks: &[Block]) {
        self.blocks.splice(
            block_index as usize..block_index as usize,
            blocks.iter().cloned(),
        );
    }

    pub fn remove_block(&mut self, block_index: i64) {
        self.blocks.remove(block_index as usize);
    }

    pub fn remove_blocks(&mut self, block_index: i64, count: i64) {
        self.blocks
            .drain(block_index as usize..block_index as usize + count as usize);
    }

    pub fn find_block(&self, index: CounterExpr) -> (i64, CounterExpr) {
        let mut work = CounterExpr::Constant(0);
        for (i, block) in self.blocks.iter().enumerate() {
            if work <= index && index < work + block.size() {
                return (i as i64, index - work);
            }
            work += block.size();
        }
        panic!("Block not found for the given index");
    }

    pub fn split_at(&mut self, index: CounterExpr) -> Option<[Vec<Block>; 2]> {
        // ブロック位置を見つける
        let (block_index, block_rem) = self.find_block(index);

        // ブロックの残り部分が0であれば分割の必要はない
        if block_rem == CounterExpr::Constant(0) {
            return None;
        }

        // ブロックを分割する
        let block = &mut self.blocks[block_index as usize];
        let left_right = block.split_at(block_rem)?;

        // 該当のブロックを削除して挿入する
        self.remove_block(block_index);
        self.insert_blocks(block_index, &left_right.concat());
        Some(left_right)
    }

    pub fn split_range(&mut self, range: Range) -> Vec<Block> {
        let start_index = range.start;
        let end_index = range.end;

        // 区切りを分割
        self.split_at(start_index);
        if self.size() > end_index + CounterExpr::Constant(1) {
            self.split_at(end_index + CounterExpr::Constant(1));
        }

        // indexを取得
        let (start_block_index, start_block_rem) = self.find_block(start_index);
        let (end_block_index, end_block_rem) = self.find_block(end_index);
        assert!(start_block_rem == CounterExpr::Constant(0));
        assert!(end_block_rem + 1 == self.blocks[end_block_index as usize].size());

        // 指定範囲を返す
        self.blocks[start_block_index as usize..=end_block_index as usize].to_vec()
    }
}
