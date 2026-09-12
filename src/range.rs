use crate::counter::CounterExpr;
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: CounterExpr,
    pub end: CounterExpr,
}

impl Range {
    pub fn reversed(&self) -> Self {
        Self {
            start: self.end * -1,
            end: self.start * -1,
        }
    }
}

impl Add for Range {
    type Output = Range;

    fn add(self, other: Self) -> Self {
        Self {
            start: self.start + other.start,
            end: self.end + other.end,
        }
    }
}

impl Add<CounterExpr> for Range {
    type Output = Range;

    fn add(self, other: CounterExpr) -> Self {
        Self {
            start: self.start + other,
            end: self.end + other,
        }
    }
}
