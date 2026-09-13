use crate::base::Symbol;

#[derive(Clone, PartialEq, Eq)]
pub struct SymbolsBlock {
    pub(crate) symbols: Vec<Symbol>,
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

impl std::fmt::Display for SymbolsBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for symbol in &self.symbols {
            write!(
                f,
                "{}",
                match symbol {
                    Symbol::Zero => '0',
                    Symbol::One => '1',
                }
            )?;
        }
        Ok(())
    }
}
