use ast::expr::Expr;
use std::vec::Vec;

#[derive(Debug)]
pub enum Stmt {
    Empty,
    Expression(Expr),
    Print(Expr),
    Declaration(String, Option<Expr>),
    Block(Vec<Stmt>),
}

pub trait Visitor<T> {
    /// Visit a statement
    fn visit_stmt(&mut self, e: &Stmt) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, v: &mut Visitor<T>) -> T { v.visit_stmt(self) }
}
