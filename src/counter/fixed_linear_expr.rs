use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Rem, Sub, SubAssign};

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
            coefficient: self.coefficient.div_euclid(rhs),
            constant: self.constant.div_euclid(rhs),
        }
    }
}

impl Rem<i64> for FixedLinearExpr {
    type Output = FixedLinearExpr;

    fn rem(self, rhs: i64) -> Self {
        FixedLinearExpr {
            coefficient: self.coefficient.rem_euclid(rhs),
            constant: self.constant.rem_euclid(rhs),
        }
    }
}

impl From<i64> for FixedLinearExpr {
    fn from(value: i64) -> Self {
        FixedLinearExpr {
            coefficient: 0,
            constant: value,
        }
    }
}

impl std::fmt::Display for FixedLinearExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.coefficient == 0 {
            return write!(f, "{}", self.constant);
        }
        match self.coefficient {
            1 => write!(f, "n")?,
            -1 => write!(f, "-n")?,
            value => write!(f, "{value}n")?,
        }
        if self.constant != 0 {
            write!(f, "{:+}", self.constant)?;
        }
        Ok(())
    }
}
