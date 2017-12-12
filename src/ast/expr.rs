use ast::token::Token;
use Boxer;

/// An Expression Node
///
/// All `Expr` types can be evaluated to a `Literal`.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Expr {
    Identifier(Token),
    Literal(Token),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Assignment(Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
}

/// Implements the visitor pattern
///
/// An implementor of Visitor<T> should recursively walk
/// a `Expr` and returns `T`.
pub trait Visitor<T> {
    /// Visit an expression
    fn visit_expr(&mut self, e: &Expr) -> T;
}

impl Expr {
    pub fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_expr(self)
    }
}

impl Boxer for Expr {
    fn boxed(self) -> Box<Self> { Box::new(self) }
}
