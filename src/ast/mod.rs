//! A module describing the Lox abstract syntax tree.

mod expr;
mod printer;

use Boxer;

pub use ast::expr::Expr;
pub use ast::printer::Printer;

/// Implements the visitor pattern
///
/// An implementor of Visitor<T> should recursively walk
/// a `Node` and returns `T`.
pub trait Visitor<T> {
    /// Visit an expression node
    fn visit_expr(&mut self, e: &Expr) -> T;
}

/// Implements an AST Node
///
/// Nodes are receptive to any `Visitor`.
pub trait Node {
    /// Accepts a Visitor to walk the Node
    fn accept<T>(&self, v: &mut Visitor<T>) -> T;
}

impl<T: Node> Boxer for T {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
