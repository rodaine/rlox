use std::ops;
use std::cmp;
use std::f64::NAN;
use std::result;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    MustBeANumber
}

pub type Result<T = Value> = result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
}

impl Value {
    pub fn any(&self) -> Result<()> { Ok(()) }

    pub fn both_any(&self, _ : &Self) -> Result<()> { Ok(()) }

    pub fn is_number(&self) -> Result<()> {
        match self {
            Value::Number(_) => Ok(()),
            _ => Err(Error::MustBeANumber),
        }
    }

    pub fn both_numbers(lhs: &Self, rhs: &Self) -> Result<()> {
        lhs.is_number()?;
        rhs.is_number()
    }

    pub fn equal(lhs: Self, rhs: Self) -> Self {
        Value::Bool(lhs == rhs)
    }

    pub fn less(lhs: Self, rhs: Self) -> Self {
        Value::Bool(lhs.lt(&rhs))
    }

    pub fn greater(lhs: Self, rhs: Self) -> Self {
        Value::Bool(lhs.gt(&rhs))
    }
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> <Self as ops::Add<Self>>::Output {
        use crate::value::Value::*;

        match (self, rhs) {
            (Number(a), Number(b)) => Number(a + b),
            _ => unreachable!(),
        }
    }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Value) -> <Self as ops::Sub<Self>>::Output {
        use crate::value::Value::*;

        match (self, rhs) {
            (Number(a), Number(b)) => Number(a - b),
            _ => unreachable!(),
        }
    }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> <Self as ops::Mul<Self>>::Output {
        use crate::value::Value::*;

        match (self, rhs) {
            (Number(a), Number(b)) => Number(a * b),
            _ => unreachable!(),
        }
    }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> <Self as ops::Mul<Self>>::Output {
        use crate::value::Value::*;

        match (self, rhs) {
            (Number(_), Number(b)) if b == 0.0 => Number(NAN),
            (Number(a), Number(b)) => Number(a / b),
            _ => unreachable!(),
        }
    }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> <Self as ops::Neg>::Output {
        use crate::value::Value::*;

        match self {
            Number(a) => Number(-a),
            _ => unreachable!(),
        }
    }
}

impl ops::Not for Value {
    type Output = Self;

    fn not(self) -> <Self as ops::Not>::Output {
        use self::Value::*;

        match self {
            Nil => self,
            Bool(x) => Bool(!x),
            Number(x) => Bool(x == 0.0),
        }
    }
}

impl cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use self::Value::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Number(l), Number(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            _ => false,
        }
    }
}

impl cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use self::Value::*;

        if let (Number(l), Number(r)) = (self, other) {
            l.partial_cmp(r)
        } else {
            None
        }
    }
}
