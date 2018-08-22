use std::ops;

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Number(f64),
    NaN,
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> <Self as ops::Add<Self>>::Output {
        use value::Value::*;

        match (self, rhs) {
            (Number(a), Number(b)) => Number(a + b),
            (Number(_), NaN) | (NaN, Number(_)) => NaN,
            (NaN, NaN) => NaN,
        }
    }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Value) -> <Self as ops::Sub<Self>>::Output {
        use value::Value::*;

        match (self, rhs) {
            (Number(a), Number(b)) => Number(a - b),
            (Number(_), NaN) | (NaN, Number(_)) => NaN,
            (NaN, NaN) => NaN,
        }
    }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> <Self as ops::Mul<Self>>::Output {
        use value::Value::*;

        match (self, rhs) {
            (Number(a), Number(b)) => Number(a * b),
            (Number(_), NaN) | (NaN, Number(_)) => NaN,
            (NaN, NaN) => NaN,
        }
    }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> <Self as ops::Mul<Self>>::Output {
        use value::Value::*;

        match (self, rhs) {
            (Number(_), Number(b)) if b == 0.0 => NaN,
            (Number(a), Number(b)) => Number(a / b),
            (Number(_), NaN) | (NaN, Number(_)) => NaN,
            (NaN, NaN) => NaN,
        }
    }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> <Self as ops::Neg>::Output {
        use value::Value::*;

        match self {
            Number(a) => Number(-a),
            NaN => NaN,
        }
    }
}
