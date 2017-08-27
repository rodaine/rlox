use ast::expr::Expr;
use std::vec::Vec;
use Boxer;

#[derive(Debug)]
pub enum Stmt {
    Empty,
    Break(u64),
    Expression(Expr),
    Print(Expr),
    Declaration(String, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}

pub trait Visitor<T> {
    /// Visit a statement
    fn visit_stmt(&mut self, e: &Stmt) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, v: &mut Visitor<T>) -> T { v.visit_stmt(self) }
}

impl Boxer for Stmt {
    fn boxed(self) -> Box<Self> { Box::new(self) }
}
