use token;
use ast::{Visitor, Node, Printer};
use std::fmt::{Debug, Formatter, Result};

/// An Expression Node
///
/// All `Expr` types can be evaluated to a `Literal`.
pub enum Expr {
    Literal(token::Literal),
    Grouping(Box<Expr>),
    Unary(token::Token, Box<Expr>),
    Binary(Box<Expr>, token::Token, Box<Expr>),
}

impl Node for Expr {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_expr(self)
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", Printer{}.visit_expr(self))
    }
}
