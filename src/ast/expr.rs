use ast::token::Token;
use Boxer;

/// An Expression Node
///
/// All `Expr` types can be evaluated to a `Literal`.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Expr {
    Identifier(Token),
    Literal(Token),
    This(Token),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Assignment(Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Get(Box<Expr>, Token),
    Set(Box<Expr>, Token, Box<Expr>),
    Super(Token, Token),
}

/// Implements the visitor pattern
///
/// An implementor of Visitor<T> should recursively walk
/// a `Expr` and returns `T`.
pub trait Visitor<T> {
    /// Visit an expression
    fn visit_expr(&mut self, _expr: &Expr) -> T { unimplemented!() }

    fn visit_identifier(&mut self, _expr: &Expr, _id: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_literal(&mut self, _expr: &Expr, _lit: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_grouping(&mut self, _expr: &Expr, _inside: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_unary(&mut self, _expr: &Expr, _op: &Token, _rhs: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_binary(&mut self, _expr: &Expr, _lhs: &Expr, _op: &Token, _rhs: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_assignment(&mut self, _expr: &Expr, _id: &Token, _val: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_call(&mut self, _expr: &Expr, _callee: &Expr, _paren: &Token, _args: &[Expr]) -> T {
        self.visit_expr(_expr)
    }

    fn visit_get(&mut self, _expr: &Expr, _callee: &Expr, _prop: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_set(&mut self, _expr: &Expr, _settee: &Expr, _prop: &Token, _val: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_this(&mut self, _expr: &Expr, _tkn: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_super(&mut self, _expr: &Expr, _tkn: &Token, _method: &Token) -> T {
        self.visit_expr(_expr)
    }
}

impl Expr {
    pub fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        use ast::expr::Expr::*;

        match *self {
            Identifier(ref id) =>
                v.visit_identifier(self, id),
            Literal(ref lit) =>
                v.visit_literal(self, lit),
            This(ref tkn) =>
                v.visit_this(self, tkn),
            Grouping(ref inside) =>
                v.visit_grouping(self, inside.as_ref()),
            Unary(ref op, ref rhs) =>
                v.visit_unary(self, op, rhs.as_ref()),
            Binary(ref lhs, ref op, ref rhs) =>
                v.visit_binary(self, lhs.as_ref(), op, rhs.as_ref()),
            Assignment(ref id, ref val) =>
                v.visit_assignment(self, id, val.as_ref()),
            Call(ref callee, ref paren, ref args) =>
                v.visit_call(self, callee.as_ref(), paren, args),
            Get(ref callee, ref prop) =>
                v.visit_get(self, callee.as_ref(), prop),
            Set(ref settee, ref prop, ref val) =>
                v.visit_set(self, settee.as_ref(), prop, val.as_ref()),
            Super(ref tkn, ref method) =>
                v.visit_super(self, tkn, method),
        }
    }
}

impl Boxer for Expr {}
