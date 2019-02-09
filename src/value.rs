use std::ops;
use std::cmp;
use std::f64::NAN;
use std::result;
use crate::token::Lexeme;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    MustBeANumber
}

pub type Result<T = Value> = result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Object {
    String(Lexeme)
}

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    Obj(Object),
}

impl Value {
    pub fn any(&self) -> Result<()> { Ok(()) }

    pub fn both_any(&self, _: &Self) -> Result<()> { Ok(()) }

    pub fn is_number(&self) -> Result<()> {
        use self::Value::Number;
        match self {
            Number(_) => Ok(()),
            _ => Err(Error::MustBeANumber),
        }
    }

    pub fn both_numbers(lhs: &Self, rhs: &Self) -> Result<()> {
        lhs.is_number()?;
        rhs.is_number()
    }

    pub fn equals(&self, rhs: &Self) -> Self { self.eq(rhs).into() }

    pub fn less_than(&self, rhs: &Self) -> Self { self.lt(rhs).into() }

    pub fn greater_than(&self, rhs: &Self) -> Self { self.gt(rhs).into() }

    pub fn is_not(&self) -> Self {
        use self::Value::*;
        use self::Object;

        match self {
            Nil => self.clone(),
            Bool(ref x) => (!*x).into(),
            Number(ref x) => (*x == 0.0).into(),
            Obj(Object::String(ref lex)) => (lex.value() == "").into(),
        }
    }

    pub fn negate(&self) -> Self {
        use crate::value::Value::Number;

        if let Number(n) = self {
            return (-*n).into();
        }

        unreachable!()
    }

    pub fn divide(&self, rhs: &Self) -> Self {
        use crate::value::Value::Number;

        match (self, rhs) {
            (Number(_), Number(b)) if *b == 0.0 => NAN.into(),
            (Number(a), Number(b)) => (*a / *b).into(),
            _ => unreachable!(),
        }
    }

    pub fn multiply(&self, rhs: &Self) -> Self {
        use crate::value::Value::Number;

        match (self, rhs) {
            (Number(a), Number(b)) => (*a * *b).into(),
            _ => unreachable!(),
        }
    }

    pub fn add(&self, rhs: &Self) -> Self {
        use crate::value::Value::*;

        match (self, rhs) {
            (Number(a), Number(b)) => (*a + *b).into(),
            (Obj(Object::String(l)), Obj(Object::String(r))) => {
                Lexeme::from_str([l.value(), r.value()].concat()).into()
            }
            _ => unreachable!(),
        }
    }

    pub fn subtract(&self, rhs: &Self) -> Self {
        use crate::value::Value::Number;

        match (self, rhs) {
            (Number(a), Number(b)) => (*a - *b).into(),
            _ => unreachable!(),
        }
    }

    pub fn into_lex(self) -> Lexeme {
        match self {
            Value::Obj(Object::String(lex)) => lex,
            _ => panic!("expected string"),
        }
    }

    pub fn lex(&self) -> &Lexeme {
        match self {
            Value::Obj(Object::String(lex)) => lex,
            _ => panic!("expected string"),
        }
    }
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> <Self as ops::Add>::Output { (&self).add(&rhs) }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> <Self as ops::Sub>::Output { (&self).subtract(&rhs) }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> <Self as ops::Mul>::Output { (&self).multiply(&rhs) }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> <Self as ops::Div>::Output { (&self).divide(&rhs) }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> <Self as ops::Neg>::Output { (&self).negate() }
}

impl ops::Not for Value {
    type Output = Self;

    fn not(self) -> <Self as ops::Not>::Output { (&self).is_not() }
}

impl cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use self::Value::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Number(l), Number(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            (Obj(Object::String(l)), Obj(Object::String(r))) => l.value() == r.value(),
            _ => false,
        }
    }
}

impl cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use self::Value::Number;

        if let (Number(l), Number(r)) = (self, other) {
            l.partial_cmp(r)
        } else {
            None
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self { Value::Bool(b) }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self { Value::Number(n) }
}

impl From<Lexeme> for Value {
    fn from(l: Lexeme) -> Self { Value::Obj(Object::String(l)) }
}

impl From<&Lexeme> for Value {
    fn from(l: &Lexeme) -> Self { l.clone().into() }
}
