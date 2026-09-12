use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Rem, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, Eq)]
pub enum CounterExpr {
    Constant(i64),
    LinearFixed(FixedLinearExpr),
    // 将来さらに増えるかも
}

impl CounterExpr {
    pub fn as_constant(&self) -> Option<i64> {
        match self {
            CounterExpr::Constant(c) => Some(*c),
            CounterExpr::LinearFixed(l) => {
                if l.coefficient != 0 {
                    return None;
                }
                Some(l.constant)
            }
            _ => None,
        }
    }

    pub fn as_linear_fixed(&self) -> Option<FixedLinearExpr> {
        match self {
            CounterExpr::LinearFixed(l) => Some(*l),
            CounterExpr::Constant(c) => Some(FixedLinearExpr {
                coefficient: 0,
                constant: *c,
            }),
            _ => None,
        }
    }
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

impl Add<i64> for CounterExpr {
    type Output = Self;

    fn add(self, other: i64) -> Self {
        self + CounterExpr::Constant(other)
    }
}

impl AddAssign<i64> for CounterExpr {
    fn add_assign(&mut self, other: i64) {
        *self = (*self) + other;
    }
}

impl Sub<i64> for CounterExpr {
    type Output = Self;

    fn sub(self, other: i64) -> Self {
        self - CounterExpr::Constant(other)
    }
}

impl SubAssign<i64> for CounterExpr {
    fn sub_assign(&mut self, other: i64) {
        *self = (*self) - other;
    }
}

impl Mul<i64> for CounterExpr {
    type Output = Self;

    fn mul(self, other: i64) -> Self {
        self * CounterExpr::Constant(other)
    }
}

impl MulAssign<i64> for CounterExpr {
    fn mul_assign(&mut self, other: i64) {
        *self = (*self) * other;
    }
}
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    fn mul(self, other: i64) -> Self {
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

impl Div<i64> for FixedLinearExpr {
    type Output = FixedLinearExpr;

    fn div(self, rhs: i64) -> Self {
        FixedLinearExpr {
            coefficient: self.coefficient / rhs,
            constant: self.constant / rhs,
        }
    }
}

impl Rem<i64> for FixedLinearExpr {
    type Output = FixedLinearExpr;

    fn rem(self, rhs: i64) -> Self {
        FixedLinearExpr {
            coefficient: self.coefficient % rhs,
            constant: self.constant % rhs,
        }
    }
}

impl Sum for CounterExpr {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(CounterExpr::Constant(0), |acc, x| acc + x)
    }
}

impl Ord for CounterExpr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_linear_fixed().cmp(&other.as_linear_fixed())
    }
}

impl PartialOrd for CounterExpr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CounterExpr {
    fn eq(&self, other: &Self) -> bool {
        self.as_linear_fixed() == other.as_linear_fixed()
    }
}

impl std::hash::Hash for CounterExpr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.as_linear_fixed(), state);
    }
}
