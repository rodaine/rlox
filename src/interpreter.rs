use std::ops::Deref;
use std::cmp::PartialOrd;

use ast::expr::{Expr, Visitor as ExprVisitor};
use ast::stmt::{Stmt, Visitor as StmtVisitor};
use result::{Result, Error};
use token::{Token, Literal};
use env::Env;
use std::rc::Rc;

#[derive(Default)]
pub struct Interpreter {
    env: Rc<Env>,
    repl: bool,
}

impl Interpreter {
    pub fn new(repl: bool) -> Self { Interpreter { env: Env::new(None), repl: repl } }
    pub fn interpret(&mut self, s: &Stmt) -> Result<()> { s.accept(self) }

    fn scoped(&self) -> Self {
        Interpreter {
            env: Env::new(Some(self.env.clone())),
            repl: false,
        }
    }
}

impl ExprVisitor<Result<Literal>> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> Result<Literal> {
        use ast::expr::Expr::*;
        use ast::expr::Expr::Literal as ExprLit;

        match *e {
            Identifier(ref id) => self.visit_ident(id.to_owned()),
            ExprLit(ref l) => Ok(l.clone()),
            Grouping(ref b) => self.evaluate(b.deref()),
            Unary(ref op, ref r) => self.visit_unary(op, r),
            Binary(ref l, ref op, ref r) => self.visit_binary(l, op, r),
            Assignment(ref id, ref r) => self.visit_assignment(id.to_owned(), r),
        }
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_stmt(&mut self, s: &Stmt) -> Result<()> {
        use ast::stmt::Stmt::*;

        match *s {
            Empty => Ok(()),
            Print(ref e) => self.visit_print_stmt(e),
            Expression(ref e) => self.visit_expr_stmt(e),
            Declaration(ref n, ref e) => self.visit_decl(n.to_owned(), e.as_ref()),
            Block(ref stmts) => self.visit_block(stmts),
            If(ref c, ref t, ref e) => self.visit_if(c, t.as_ref(), e.as_ref().map(|x| x.deref())),
            While(ref e, ref b) => self.visit_while(e, b.deref()),
            Break(l) => self.visit_break(l),
        }
    }
}

// Private, expression-related methods
impl Interpreter {
    fn visit_expr_stmt(&mut self, e: &Expr) -> Result<()> {
        if self.repl {
            self.visit_print_stmt(e)
        } else {
            e.accept(self).map(|_| ())
        }
    }

    fn visit_print_stmt(&mut self, e: &Expr) -> Result<()> {
        println!("{}", e.accept(self)?);
        Ok(())
    }

    fn visit_decl(&mut self, name: String, init: Option<&Expr>) -> Result<()> {
        let val: Literal = init.map_or_else(|| Ok(Literal::Nil), |e| e.accept(self))?;
        self.env.define(&name, val)
    }

    fn visit_block(&mut self, stmts: &[Stmt]) -> Result<()> {
        let mut scope: Self = self.scoped();
        for stmt in stmts { stmt.accept(&mut scope)?; }
        Ok(())
    }

    fn visit_if(&mut self, expr: &Expr, then_stmt: &Stmt, else_stmt: Option<&Stmt>) -> Result<()> {
        let cond: Literal = expr.accept(self)?;

        if cond.is_truthy() {
            return then_stmt.accept(self);
        }

        if else_stmt.is_none() {
            Ok(())
        } else {
            else_stmt.unwrap().deref().accept(self)
        }
    }

    fn visit_while(&mut self, expr: &Expr, body: &Stmt) -> Result<()> {
        while self.evaluate(expr)?.is_truthy() {
            match body.accept(self) {
                Err(Error::Break(_)) => break,
                Err(e) => return Err(e),
                Ok(_) => (),
            };
        }

        Ok(())
    }

    fn visit_break(&mut self, line: u64) -> Result<()> {
        Err(Error::Break(line))
    }
}

// Private, expression-related methods
impl Interpreter {
    fn evaluate(&mut self, b: &Expr) -> Result<Literal> { b.accept(self) }

    fn visit_unary(&mut self, op: &Token, rhs: &Expr) -> Result<Literal> {
        use token::Type::{Minus, Bang};
        use token::Literal::*;

        let r: Literal = self.evaluate(rhs)?;

        match op.typ {
            Minus => match r {
                Literal::Number(n) => Ok(Literal::Number(-n)),
                _ => self.err_near("cannot negate non-numeric", op, format!("{:?}", r)),
            },
            Bang => Ok(Boolean(!r.is_truthy())),
            _ => self.err_op("erroneous unary operator", op),
        }
    }

    fn visit_binary(&mut self, lhs: &Expr, op: &Token, rhs: &Expr) -> Result<Literal> {
        use token::Type::{Plus, Minus, Star, Slash, Greater, GreaterEqual, Less, LessEqual, EqualEqual, BangEqual, Or, And};
        use std::cmp::Ordering as Ord;
        use token::Literal::*;

        if op.in_types(&[Or, And]) {
            return self.visit_logical(lhs, op, rhs);
        }

        let l: Literal = self.evaluate(lhs)?;
        let r: Literal = self.evaluate(rhs)?;

        match op.typ {
            Plus => match (self.evaluate(lhs)?, self.evaluate(rhs)?) {
                (Number(ln), Number(rn)) => Ok(Number(ln + rn)),
                (String(ln), r) => Ok(String(format!("{}{}", ln, r))),
                (l, String(rn)) => Ok(String(format!("{}{}", l, rn))),
                (l, r) => self.err_near("cannot add mixed types", op, format!("{:?} + {:?}", l, r)),
            },
            Minus => match (self.evaluate(lhs)?, self.evaluate(rhs)?) {
                (Number(ln), Number(rn)) => Ok(Number(ln - rn)),
                (l, r) => self.err_near("cannot subtract non-numerics", op, format!("{:?} - {:?}", l, r)),
            },
            Star => match (self.evaluate(lhs)?, self.evaluate(rhs)?) {
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

    fn visit_logical(&mut self, lhs: &Expr, op: &Token, rhs: &Expr) -> Result<Literal> {
        use token::Type::{Or, And};
        use token::Literal::Boolean;

        let l: Literal = self.evaluate(lhs)?;

        match op.typ {
            And if l.is_truthy() => Ok(Boolean(self.evaluate(rhs)?.is_truthy())),
            Or if l.is_truthy() => Ok(Boolean(true)),
            Or => Ok(Boolean(self.evaluate(rhs)?.is_truthy())),
            _ => Ok(Boolean(false)),
        }
    }

    fn visit_ident(&mut self, n: String) -> Result<Literal> {
        self.env.get(&n).map(|lit| lit.clone())
    }

    fn visit_assignment(&mut self, n: String, rhs: &Expr) -> Result<Literal> {
        let val = self.evaluate(rhs)?;
        self.env.assign(&n, val).map(|lit| lit.clone())
    }
}

impl Interpreter {
    fn err_op(&self, msg: &str, op: &Token) -> Result<Literal> {
        Err(Error::Runtime(
            op.line,
            msg.to_string(),
            op.lexeme.clone(),
        ))
    }

    fn err_near(&self, msg: &str, op: &Token, near: String) -> Result<Literal> {
        Err(Error::Runtime(
            op.line,
            msg.to_string(),
            near,
        ))
    }
}
