use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CounterExpr {
    Constant(i64),
    LinearFixed(FixedLinearExpr),
    // 将来さらに増えるかも
}

impl Add for CounterExpr {
    type Output = Self;

    fn add(self, other: CounterExpr) -> CounterExpr {
        match (self, other) {
            (CounterExpr::Constant(a), CounterExpr::Constant(b)) => CounterExpr::Constant(a + b),
            (CounterExpr::Constant(a), CounterExpr::LinearFixed(b)) => {
                CounterExpr::LinearFixed(b + a)
            }
            (CounterExpr::LinearFixed(a), CounterExpr::Constant(b)) => {
                CounterExpr::LinearFixed(a + b)
            }
            (CounterExpr::LinearFixed(a), CounterExpr::LinearFixed(b)) => {
                CounterExpr::LinearFixed(a + b)
            }
        }
    }
}

impl AddAssign for CounterExpr {
    fn add_assign(&mut self, other: CounterExpr) {
        *self = (*self) + other;
    }
}

impl Sub for CounterExpr {
    type Output = Self;
    fn sub(self, other: CounterExpr) -> CounterExpr {
        match (self, other) {
            (CounterExpr::Constant(a), CounterExpr::Constant(b)) => CounterExpr::Constant(a - b),
            (CounterExpr::Constant(a), CounterExpr::LinearFixed(b)) => {
                CounterExpr::LinearFixed(b * -1 + a)
            }
            (CounterExpr::LinearFixed(a), CounterExpr::Constant(b)) => {
                CounterExpr::LinearFixed(a + -b)
            }
            (CounterExpr::LinearFixed(a), CounterExpr::LinearFixed(b)) => {
                CounterExpr::LinearFixed(a + b * -1)
            }
        }
    }
}

impl SubAssign for CounterExpr {
    fn sub_assign(&mut self, other: CounterExpr) {
        *self = (*self) - other;
    }
}

impl Mul for CounterExpr {
    type Output = Self;
    fn mul(self, other: CounterExpr) -> CounterExpr {
        match (self, other) {
            (CounterExpr::Constant(a), CounterExpr::Constant(b)) => CounterExpr::Constant(a * b),
            (CounterExpr::Constant(a), CounterExpr::LinearFixed(b)) => {
                CounterExpr::LinearFixed(b * a)
            }
            (CounterExpr::LinearFixed(a), CounterExpr::Constant(b)) => {
                CounterExpr::LinearFixed(a * b)
            }
            _ => unimplemented!(),
        }
    }
}

impl MulAssign for CounterExpr {
    fn mul_assign(&mut self, other: CounterExpr) {
        *self = (*self) * other;
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedLinearExpr {
    pub coefficient: i64,
    pub constant: i64,
}

impl FixedLinearExpr {
    pub fn from_i64(other: i64) -> Self {
        Self {
            coefficient: 0,
            constant: other,
        }
    }
}

impl Add for FixedLinearExpr {
    type Output = Self;

    fn add(self, other: FixedLinearExpr) -> FixedLinearExpr {
        FixedLinearExpr {
            coefficient: self.coefficient + other.coefficient,
            constant: self.constant + other.constant,
        }
    }
}

impl Add<i64> for FixedLinearExpr {
    type Output = Self;

    fn add(self, other: i64) -> Self {
        FixedLinearExpr {
            coefficient: self.coefficient,
            constant: self.constant + other,
        }
    }
}

impl AddAssign for FixedLinearExpr {
    fn add_assign(&mut self, other: FixedLinearExpr) {
        *self = (*self) + other;
    }
}

impl AddAssign<i64> for FixedLinearExpr {
    fn add_assign(&mut self, other: i64) {
        *self = (*self) + other;
    }
}

impl Sub for FixedLinearExpr {
    type Output = Self;

    fn sub(self, other: FixedLinearExpr) -> FixedLinearExpr {
        FixedLinearExpr {
            coefficient: self.coefficient - other.coefficient,
            constant: self.constant - other.constant,
        }
    }
}

impl Sub<i64> for FixedLinearExpr {
    type Output = Self;

    fn sub(self, other: i64) -> Self {
        FixedLinearExpr {
            coefficient: self.coefficient,
            constant: self.constant - other,
        }
    }
}

impl SubAssign for FixedLinearExpr {
    fn sub_assign(&mut self, other: Self) {
        *self = (*self) - other;
    }
}

impl SubAssign<i64> for FixedLinearExpr {
    fn sub_assign(&mut self, other: i64) {
        *self = (*self) - other;
    }
}

impl Mul<i64> for FixedLinearExpr {
    type Output = Self;

    fn mul(self, other: i64) -> FixedLinearExpr {
        FixedLinearExpr {
            coefficient: self.coefficient * other,
            constant: self.constant * other,
        }
    }
}

impl MulAssign<i64> for FixedLinearExpr {
    fn mul_assign(&mut self, other: i64) {
        *self = (*self) * other;
    }
}
