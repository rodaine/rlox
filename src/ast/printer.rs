use ast::Expr;
use ast::Expr::*;
use ast::{Visitor, Node};
use std::ops::Deref;

/// "Pretty" prints the AST nodes and implements the `Visitor` trait.
///
/// This printer is utilized by the AST `Node` types for their `fmt::Debug` implementations
///
/// # Examples
/// ```
/// # extern crate rlox;
/// # use rlox::token::Token;
/// # use rlox::token::Literal::*;
/// # use rlox::ast::{Printer, Visitor};
/// # use rlox::ast::Expr::*;
/// # use rlox::Boxer;
/// # fn main() {
/// let minus = Token{lexeme: "-".to_string(), ..Token::default() };
/// let times = Token{lexeme: "*".to_string(), ..Token::default() };
///
/// let e = Binary(
///     Unary(minus, Literal(Number(123f64)).boxed()).boxed(),
///     times,
///     Grouping(Literal(Number(45.67f64)).boxed()).boxed()
/// );
///
/// let mut p = Printer;
///
/// assert_eq!(
///     "(* (- 123) (group 45.67))",
///     Printer.visit_expr(&e)
/// )
/// # }
/// ```
pub struct Printer;

impl Printer {
    fn parens(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut s = String::new();

        s.push('(');
        s.push_str(name);

        for ex in exprs {
            s.push(' ');
            s.push_str(ex.accept(self).deref());
        }

        s.push(')');

        s
    }
}

impl Visitor<String> for Printer {
    fn visit_expr(&mut self, e: &Expr) -> String {
        use token::Literal::{Number as Num, String as Str, Nil as Null, Boolean as Bln};

        match *e {
            Literal(Num(n)) => format!("{}", n),
            Literal(Str(ref s)) => format!("\"{}\"", s),
            Literal(Null) => String::from("nil"),
            Literal(Bln(b)) => format!("{}", b),

            Grouping(ref e) => self.parens("group", &[e.deref()]),

            Unary(ref op, ref e) => self.parens(op.lexeme.deref(), &[e.deref()]),
            Binary(ref l, ref op, ref r) => self.parens(op.lexeme.deref(), &[l.deref(), r.deref()]),

            // uncomment if not exhaustive
            // _ => String::from("UNKNOWN"),
        }
    }
}
