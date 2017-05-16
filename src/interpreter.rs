use std::ops::Deref;
use std::cmp::PartialOrd;

use ast::{Visitor, Expr, Node};
use result::{Result, Error};
use token::{Token, Literal};

pub struct Interpreter;

impl Interpreter {
    pub fn run<T: Node>(n: &T) -> Result<Literal> { n.accept(&mut Interpreter {}) }
}

impl Visitor<Result<Literal>> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> Result<Literal> {
        use ast::Expr::*;
        use ast::Expr::Literal as ExprLit;

        match *e {
            ExprLit(ref l) => Ok(l.clone()),
            Grouping(ref b) => self.evaluate(b),
            Unary(ref op, ref r) => self.visit_unary(op, r),
            Binary(ref l, ref op, ref r) => self.visit_binary(l, op, r),
        }
    }
}

impl Interpreter {
    fn evaluate(&mut self, b: &Box<Expr>) -> Result<Literal> { b.deref().accept(self) }

    fn visit_unary(&mut self, op: &Token, rhs: &Box<Expr>) -> Result<Literal> {
        use token::Type::{Minus, Bang};
        use token::Literal::*;

        let r: Literal = self.evaluate(rhs)?;

        match op.typ {
            Minus => match r {
                Literal::Number(n) => Ok(Literal::Number(-n)),
                _ => self.err_near("cannot negate non-numeric", op, format!("{:?}", r)),
            },
            Bang => Ok(Boolean(match r {
                Nil => false,
                String(ref s) => !s.is_empty(),
                Boolean(b) => b,
                Number(n) => n > 0.0,
            })),
            _ => self.err_op("erroneous unary operator", op),
        }
    }

    fn visit_binary(&mut self, lhs: &Box<Expr>, op: &Token, rhs: &Box<Expr>) -> Result<Literal> {
        use token::Type::{Plus, Minus, Star, Slash, Greater, GreaterEqual, Less, LessEqual, EqualEqual, BangEqual};
        use std::cmp::Ordering as Ord;
        use token::Literal::*;

        let l: Literal = self.evaluate(lhs)?;
        let r: Literal = self.evaluate(rhs)?;

        match op.typ {
            Plus => match (l, r) {
                (Number(ln), Number(rn)) => Ok(Number(ln + rn)),
                (String(ln), r) => Ok(String(format!("{}{}", ln, r))),
                (l, String(rn)) => Ok(String(format!("{}{}", l, rn))),
                (l, r) => self.err_near("cannot add mixed types", op, format!("{:?} + {:?}", l, r)),
            },
            Minus => match (l, r) {
                (Number(ln), Number(rn)) => Ok(Number(ln - rn)),
                (l, r) => self.err_near("cannot subtract non-numerics", op, format!("{:?} - {:?}", l, r)),
            },
            Star => match (l, r) {
                (Number(ln), Number(rn)) => Ok(Number(ln * rn)),
                (l, r) => self.err_near("cannot multiply non-numerics", op, format!("{:?} * {:?}", l, r)),
            },
            Slash => match (l, r) {
                (Number(ln), Number(rn)) if rn == 0.0 => self.err_near("divide by zero", op, format!("{:?} / {:?}", ln, rn)),
                (Number(ln), Number(rn)) => Ok(Number(ln / rn)),
                (l, r) => self.err_near("cannot multiply non-numerics", op, format!("{:?} * {:?}", l, r)),
            },
            Greater | GreaterEqual | Less | LessEqual => match l.partial_cmp(&r) {
                Some(Ord::Less) => Ok(Boolean(op.in_types(&[Less, LessEqual]))),
                Some(Ord::Equal) => Ok(Boolean(op.in_types(&[LessEqual, GreaterEqual]))),
                Some(Ord::Greater) => Ok(Boolean(op.in_types(&[Greater, GreaterEqual]))),
                None => self.err_near("cannot compare types", op, format!("{:?} ? {:?}", l, r)),
            },
            EqualEqual => Ok(Boolean(l.eq(&r))),
            BangEqual => Ok(Boolean(l.ne(&r))),
            _ => self.err_op("erroneous binary operator", op),
        }
    }

    fn err_op(&self, msg: &str, op: &Token) -> Result<Literal> {
        let e = Error::Runtime(
            op.line,
            msg.to_string(),
            op.lexeme.clone(),
        );

        Err(e.boxed())
    }

    fn err_near(&self, msg: &str, op: &Token, near: String) -> Result<Literal> {
        let e = Error::Runtime(
            op.line,
            msg.to_string(),
            near,
        );

        Err(e.boxed())
    }
}
