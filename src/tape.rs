use crate::block::Block;
use crate::counter::CounterExpr;
use crate::range::Range;

#[derive(Clone)]
pub struct Tape {
    blocks: Vec<Block>,
}

impl Tape {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks }
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

    pub fn blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    fn standaraized(tape: &Tape) -> Tape {
        // ここで必要な標準化処理を行う
        // 空のブロックは削除
        let mut tape = tape.clone();
        for i in (0..tape.blocks.len()).rev() {
            if tape.blocks[i].size() == CounterExpr::Constant(0) {
                tape.blocks.remove(i);
            }
        }
        tape
    }

    pub fn compare(lhs: &Tape, rhs: &Tape) -> bool {
        Self::compare_with_debug(lhs, rhs, false)
    }

    pub fn compare_with_debug(lhs: &Tape, rhs: &Tape, debug: bool) -> bool {
        // サイズを調べる
        if lhs.size() != rhs.size() {
            if debug {
                println!("    {lhs}\n    {rhs}\n");
            }
            return false;
        }

        let mut lhs = Self::standaraized(lhs);
        let mut rhs = Self::standaraized(rhs);

        // ブロックを比較する
        loop {
            if debug {
                println!("    {lhs}\n    {rhs}\n");
            }
            if lhs.blocks().is_empty() && rhs.blocks().is_empty() {
                return true;
            } else if lhs.blocks().is_empty() || rhs.blocks().is_empty() {
                return false;
            }

            // 先頭ブロックを比較する
            let lhs_first = lhs.block(0);
            let rhs_first = rhs.block(0);

            // ブロックのサイズを比較して、必要に応じて分割する
            if rhs_first.size() < lhs_first.size() {
                lhs.split_at(rhs_first.size());
                continue;
            } else if lhs_first.size() < rhs_first.size() {
                rhs.split_at(lhs_first.size());
                continue;
            }

            // サイズが等しい場合は先頭ブロックを比較
            if lhs_first != rhs_first {
                return false;
            }

            // 先頭ブロックを削除
            lhs.remove_block(0);
            rhs.remove_block(0);
        }
    }

    pub fn reversed(&self) -> Self {
        Self {
            blocks: self.blocks.iter().rev().map(|b| b.reversed()).collect(),
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

impl std::fmt::Display for Tape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.blocks.is_empty() {
            return write!(f, "(empty)");
        }
        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                write!(f, " | ")?;
            }
            write!(f, "{block}")?;
        }
        Ok(())
    }
}

impl Tape {
    /// Show the head without expanding or splitting symbolic repeat blocks.
    pub fn debug_with_position(&self, position: CounterExpr) -> String {
        let mut text = String::new();
        let mut start = CounterExpr::Constant(0);
        let mut marker = None;
        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                text.push_str(" | ");
            }
            let column = text.len();
            let end = start + block.size();
            if start <= position && position < end {
                let offset = position - start;
                let cell_column = block
                    .as_symbols()
                    .and_then(|_| offset.as_constant())
                    .map(|offset| column + 1 + offset as usize)
                    .unwrap_or(column);
                marker = Some((
                    cell_column,
                    format!("pos:{position} (block:{}, offset:{offset})", i + 1),
                ));
            }
            text.push_str(&block.to_string());
            start = end;
        }
        if text.is_empty() {
            text.push_str("(empty)");
        }
        let (column, label) = marker.unwrap_or_else(|| {
            if position < CounterExpr::Constant(0) {
                (
                    0,
                    format!("pos:{position} (left blank, distance:{})", position * -1),
                )
            } else {
                (
                    text.len(),
                    format!("pos:{position} (right blank, offset:{})", position - start),
                )
            }
        });
        format!("{text}\n          {}^ {label}", " ".repeat(column))
    }
}
