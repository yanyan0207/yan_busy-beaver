use std::ops::{Add, Sub};
pub struct FixedLinearExpr {
    coefficient: i64,
    constant: i64,
    min_n: i64,
}

impl FixedLinearExpr {
    pub fn new(coefficient: i64, constant: i64, min_n: i64) -> Self {
        FixedLinearExpr {
            coefficient,
            constant,
            min_n,
        }
    }
}

pub enum CounterExpr {
    Constant(i64),
    LinearFixed(FixedLinearExpr),
    // 将来さらに増えるかも
}

impl Add for CounterExpr {
    type Output = Self;

    pub fn add(self, other: CounterExpr) -> CounterExpr {
        match (self, other) {
            (CounterExpr::Constant(a), CounterExpr::Constant(b)) => CounterExpr::Constant(a + b),
            (CounterExpr::Constant(a), CounterExpr::LinearFixed(b)) => CounterExpr::LinearFixed(
                FixedLinearExpr::new(b.coefficient, a + b.constant, b.min_n),
            ),
            (CounterExpr::LinearFixed(a), CounterExpr::Constant(b)) => CounterExpr::LinearFixed(
                FixedLinearExpr::new(a.coefficient, a.constant + b, a.min_n),
            ),
            (CounterExpr::LinearFixed(a), CounterExpr::LinearFixed(b)) => {
                CounterExpr::LinearFixed(FixedLinearExpr::new(
                    a.coefficient + b.coefficient,
                    a.constant + b.constant,
                    a.min_n.min(b.min_n),
                ))
            }
            _ => unimplemented!(),
        }
    }
}

impl AddAssign for CounterExpr {
    fn add_assign(&mut self, other: CounterExpr) {
        *self = self.clone() + other;
    }
}

impl Sub for CounterExpr {
    type Output = Self;
    pub fn sub(self, other: CounterExpr) -> CounterExpr {
        match (self, other) {
            (CounterExpr::Constant(a), CounterExpr::Constant(b)) => CounterExpr::Constant(a - b),
            (CounterExpr::Constant(a), CounterExpr::LinearFixed(b)) => CounterExpr::LinearFixed(
                FixedLinearExpr::new(-b.coefficient, a - b.constant, b.min_n),
            ),
            (CounterExpr::LinearFixed(a), CounterExpr::Constant(b)) => CounterExpr::LinearFixed(
                FixedLinearExpr::new(a.coefficient, a.constant - b, a.min_n),
            ),
            (CounterExpr::LinearFixed(a), CounterExpr::LinearFixed(b)) => {
                CounterExpr::LinearFixed(FixedLinearExpr::new(
                    a.coefficient - b.coefficient,
                    a.constant - b.constant,
                    a.min_n.min(b.min_n),
                ))
            }
            _ => unimplemented!(),
        }
    }
}
impl MulAssign for CounterExpr {
    fn mul_assign(&mut self, other: CounterExpr) {
        *self = self.clone() * other;
    }
}

impl Mul for CounterExpr {
    type Output = Self;
    pub fn mul(self, other: CounterExpr) -> CounterExpr {
        match (self, other) {
            (CounterExpr::Constant(a), CounterExpr::Constant(b)) => CounterExpr::Constant(a * b),
            (CounterExpr::Constant(a), CounterExpr::LinearFixed(b)) => CounterExpr::LinearFixed(
                FixedLinearExpr::new(a * b.coefficient, a * b.constant, b.min_n),
            ),
            (CounterExpr::LinearFixed(a), CounterExpr::Constant(b)) => CounterExpr::LinearFixed(
                FixedLinearExpr::new(a.coefficient * b, a.constant * b, a.min_n),
            ),
            _ => unimplemented!(),
        }
    }
}

impl MulAssign for CounterExpr {
    fn mul_assign(&mut self, other: CounterExpr) {
        *self = self.clone() * other;
    }
}
