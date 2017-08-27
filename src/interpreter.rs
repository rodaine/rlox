use std::ops::Deref;
use std::cmp::PartialOrd;

use ast::expr::{Expr, Visitor as ExprVisitor};
use ast::stmt::{Stmt, Visitor as StmtVisitor};
use result::{Result, Error};
use token::{Token, Literal};
use env::Env;
use object::Object;
use std::rc::Rc;
use functions::LoxFunction;

#[derive(Default)]
pub struct Interpreter {
    pub env: Rc<Env>,
    repl: bool,
}

impl Interpreter {
    pub fn new(repl: bool) -> Self { Interpreter { env: Env::new(), repl: repl } }
    pub fn with_env(env : Rc<Env>) -> Self { Interpreter { env: env, repl: false } }

    pub fn interpret(&mut self, s: &Stmt) -> Result<()> { s.accept(self) }

    fn scoped(&self) -> Self {
        Interpreter {
            env: Env::with_parent(self.env.clone()),
            repl: false,
        }
    }
}

impl ExprVisitor<Result<Object>> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> Result<Object> {
        use ast::expr::Expr::*;
        use ast::expr::Expr::Literal as ExprLit;

        match *e {
            Identifier(ref id) => self.visit_ident(id.to_owned()),
            ExprLit(ref l) => Ok(Object::Literal(l.clone())),
            Grouping(ref b) => self.evaluate(b.deref()),
            Unary(ref op, ref r) => self.visit_unary(op, r),
            Binary(ref l, ref op, ref r) => self.visit_binary(l, op, r),
            Assignment(ref id, ref r) => self.visit_assignment(id.to_owned(), r),
            Call(ref expr, ref paren, ref body) => self.visit_call(expr, paren, body.deref()),
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
            Function(ref n, ref p, ref b) => self.visit_function(n.to_owned(), p, b.clone()),
            Return(l, ref e) => self.visit_return(l, e),
        }
    }
}

// Private, statement-related methods
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
        let val: Object = init.map_or_else(
            || Ok(Object::Literal(Literal::Nil)),
            |e| e.accept(self))?;

        self.env.define(&name, val)
    }

    fn visit_block(&mut self, stmts: &[Stmt]) -> Result<()> {
        let mut scope: Self = self.scoped();
        for stmt in stmts { stmt.accept(&mut scope)?; }
        Ok(())
    }

    fn visit_if(&mut self, expr: &Expr, then_stmt: &Stmt, else_stmt: Option<&Stmt>) -> Result<()> {
        let cond: Object = expr.accept(self)?;

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

    fn visit_function(&mut self, name: String, params: &[String], body: Rc<Stmt>) -> Result<()> {
        self.env.define(&name, Object::Func(LoxFunction::new(self.env.clone(), params, body)))
    }

    fn visit_return(&mut self, line: u64, expr: &Expr) -> Result<()> {
        let res : Object = self.evaluate(expr)?;
        Err(Error::Return(line, res))
    }
}

// Private, expression-related methods
impl Interpreter {
    fn evaluate(&mut self, b: &Expr) -> Result<Object> { b.accept(self) }

    fn visit_unary(&mut self, op: &Token, rhs: &Expr) -> Result<Object> {
        use token::Type::{Minus, Bang};
        use token::Literal::{Number, Boolean};

        let r: Object = self.evaluate(rhs)?;

        match op.typ {
            Minus => match r {
                Object::Literal(Number(n)) => Ok(Object::Literal(Number(-n))),
                _ => self.err_near("cannot negate non-numeric", op, format!("{:?}", r)),
            },
            Bang => Ok(Object::Literal(Boolean(!r.is_truthy()))),
            _ => self.err_op("erroneous unary operator", op),
        }
    }

    fn visit_binary(&mut self, lhs: &Expr, op: &Token, rhs: &Expr) -> Result<Object> {
        use token::Type::{Plus, Minus, Star, Slash, Greater, GreaterEqual, Less, LessEqual, EqualEqual, BangEqual, Or, And};
        use std::cmp::Ordering as Ord;
        use token::Literal::*;
        use object::Object::Literal as ObjLit;

        if op.in_types(&[Or, And]) {
            return self.visit_logical(lhs, op, rhs);
        }

        let l: Object = self.evaluate(lhs)?;
        let r: Object = self.evaluate(rhs)?;

        let res: Literal = match op.typ {
            Plus => match (self.evaluate(lhs)?, self.evaluate(rhs)?) {
                (ObjLit(Number(ln)), ObjLit(Number(rn))) => Number(ln + rn),
                (ObjLit(String(ln)), ObjLit(r)) => String(format!("{}{}", ln, r)),
                (ObjLit(l), ObjLit(String(rn))) => String(format!("{}{}", l, rn)),
                (l, r) => return self.err_near(
                    "cannot add mixed types",
                    op, format!("{:?} + {:?}", l, r)),
            },
            Minus => match (self.evaluate(lhs)?, self.evaluate(rhs)?) {
                (ObjLit(Number(ln)), ObjLit(Number(rn))) => Number(ln - rn),
                (l, r) => return self.err_near(
                    "cannot subtract non-numerics",
                    op, format!("{:?} - {:?}", l, r)),
            },
            Star => match (self.evaluate(lhs)?, self.evaluate(rhs)?) {
                (ObjLit(Number(ln)), ObjLit(Number(rn))) => Number(ln * rn),
                (l, r) => return self.err_near(
                    "cannot multiply non-numerics",
                    op, format!("{:?} * {:?}", l, r)),
            },
            Slash => match (l, r) {
                (ObjLit(Number(ln)), ObjLit(Number(rn))) if rn == 0.0 => return self.err_near(
                    "divide by zero",
                    op, format!("{:?} / {:?}", ln, rn)),
                (ObjLit(Number(ln)), ObjLit(Number(rn))) => Number(ln / rn),
                (l, r) => return self.err_near(
                    "cannot multiply non-numerics",
                    op, format!("{:?} * {:?}", l, r)),
            },
            Greater | GreaterEqual | Less | LessEqual => match l.partial_cmp(&r) {
                Some(Ord::Less) => Boolean(op.in_types(&[Less, LessEqual])),
                Some(Ord::Equal) => Boolean(op.in_types(&[LessEqual, GreaterEqual])),
                Some(Ord::Greater) => Boolean(op.in_types(&[Greater, GreaterEqual])),
                None => return self.err_near(
                    "cannot compare types",
                    op, format!("{:?} ? {:?}", l, r)),
            },
            EqualEqual => Boolean(l.eq(&r)),
            BangEqual => Boolean(l.ne(&r)),
            _ => return self.err_op("erroneous binary operator", op),
        };

        Ok(ObjLit(res))
    }

    fn visit_logical(&mut self, lhs: &Expr, op: &Token, rhs: &Expr) -> Result<Object> {
        use token::Type::{Or, And};
        use token::Literal::Boolean;

        let l: Object = self.evaluate(lhs)?;

        let res: Literal = match op.typ {
            And if l.is_truthy() => Boolean(self.evaluate(rhs)?.is_truthy()),
            Or if l.is_truthy() => Boolean(true),
            Or => Boolean(self.evaluate(rhs)?.is_truthy()),
            _ => Boolean(false),
        };

        Ok(Object::Literal(res))
    }

    fn visit_ident(&mut self, n: String) -> Result<Object> {
        self.env.get(&n).map(|lit| lit.clone())
    }

    fn visit_assignment(&mut self, n: String, rhs: &Expr) -> Result<Object> {
        let val = self.evaluate(rhs)?;
        self.env.assign(&n, val).map(|lit| lit.clone())
    }

    fn visit_call(&mut self, expr: &Expr, paren: &Token, params: &[Expr]) -> Result<Object> {
        let callee = match self.evaluate(expr)? {
            Object::Func(c) => c,
            x => return self.err_near(
                "can only call functions and classes",
                paren, format!("{:?}", x)),
        };

        if callee.arity() != params.len() {
            return self.err_near(
                &format!("expected {} arguments but got {}", callee.arity(), params.len()),
                paren, "".to_string())
        }

        let mut args : Vec<Object> = Vec::with_capacity(params.len());
        for param in params {
            args.push(self.evaluate(param)?);
        }

        callee.call(self, &args)
    }
}

impl Interpreter {
    fn err_op(&self, msg: &str, op: &Token) -> Result<Object> {
        Err(Error::Runtime(
            op.line,
            msg.to_string(),
            op.lexeme.clone(),
        ))
    }

    fn err_near(&self, msg: &str, op: &Token, near: String) -> Result<Object> {
        Err(Error::Runtime(
            op.line,
            msg.to_string(),
            near,
        ))
    }
}
