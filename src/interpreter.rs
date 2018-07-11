use std::cmp::PartialOrd;
use std::collections::HashMap;
use std::rc::Rc;

use ast::expr::{Expr, Visitor as ExprVisitor};
use ast::stmt::{Stmt, Visitor as StmtVisitor};
use ast::token::{Token, Literal};

use class::{LoxClass, SUPER_ID, THIS_ID};
use env::Env;
use functions::{Callable, INITIALIZER_FUNC};
use object::Object;
use result::{Result, Error};
use output::Writer;
use std::cell::RefCell;

pub struct Interpreter {
    env: Rc<Env>,
    locals: Rc<HashMap<Expr, usize>>,
    repl: bool,
    stdout: Rc<RefCell<Writer>>,
}

#[cfg(feature = "debug-destructors")]
impl Drop for Interpreter {
    fn drop(&mut self) {
        match (Rc::strong_count(&self.locals), Rc::strong_count(&self.env)) {
            (1, e) => debug_drop!("Interpreter::Root ({} env refs remaining)", e-1),
            (l, e) => debug_drop!("Interpreter::Child ({} parent & {} env refs remaining)", l-2, e-1),
        }
    }
}


impl Interpreter {
    pub fn new(repl: bool, stdout: Rc<RefCell<Writer>>) -> Interpreter {
        let i = Interpreter {
            repl,
            env: Env::new(),
            locals: Rc::new(HashMap::new()),
            stdout,
        };

        debug_create!("Interpreter::Root (REPL: {})", i.repl);

        i
    }

    pub fn with_env(&self, env: Rc<Env>) -> Interpreter {
        debug_create!("interpreter with env{}", "");
        Interpreter {
            env,
            locals: Rc::clone(&self.locals),
            repl: self.repl,
            stdout: Rc::clone(&self.stdout),
        }
    }

    pub fn resolve(&mut self, b: &Expr, idx: usize) {
        Rc::get_mut(&mut self.locals)
            .expect("should be the only ref given the &mut")
            .insert(b.clone(), idx);
    }
}

impl ExprVisitor<Result<Object>> for Interpreter {
    fn visit_identifier(&mut self, expr: &Expr, id: &Token) -> Result<Object> {
        self.lookup_var(id, expr)
    }

    fn visit_literal(&mut self, _expr: &Expr, lit: &Token) -> Result<Object> {
        Ok(Object::Literal(lit.literal.as_ref().unwrap().clone()))
    }

    fn visit_grouping(&mut self, _expr: &Expr, inside: &Expr) -> Result<Object> {
        inside.accept(self)
    }

    fn visit_unary(&mut self, _expr: &Expr, op: &Token, rhs: &Expr) -> Result<Object> {
        use ast::token::Type::{Minus, Bang};
        use ast::token::Literal::{Number, Boolean};

        let r: Object = rhs.accept(self)?;

        match op.typ {
            Minus => match r {
                Object::Literal(Number(n)) => Ok(Object::Literal(Number(-n))),
                _ => self.err_near("cannot negate non-numeric", op, format!("{:?}", r)),
            },
            Bang => Ok(Object::Literal(Boolean(!r.is_truthy()))),
            _ => self.err_op("erroneous unary operator", op),
        }
    }

    fn visit_binary(&mut self, _expr: &Expr, lhs: &Expr, op: &Token, rhs: &Expr) -> Result<Object> {
        use ast::token::Type::{Plus, Minus, Star, Slash, Greater, GreaterEqual,
                               Less, LessEqual, EqualEqual, BangEqual, Or, And};
        use std::cmp::Ordering as Ord;
        use ast::token::Literal::*;
        use object::Object::Literal as ObjLit;

        if op.in_types(&[Or, And]) {
            return self.visit_logical(lhs, op, rhs);
        }

        let l: Object = lhs.accept(self)?;
        let r: Object = rhs.accept(self)?;

        let res: Literal = match op.typ {
            Plus => match (l, r) {
                (ObjLit(Number(ref ln)), ObjLit(Number(ref rn))) => Number(ln + rn),
                (ObjLit(String(ref ln)), ObjLit(ref r)) => String(format!("{}{}", ln, r)),
                (ObjLit(ref l), ObjLit(String(ref rn))) => String(format!("{}{}", l, rn)),
                (ref l, ref r) => return self.err_near(
                    "cannot add mixed types",
                    op, format!("{:?} + {:?}", l, r)),
            },
            Minus => match (l, r) {
                (ObjLit(Number(ln)), ObjLit(Number(rn))) => Number(ln - rn),
                (l, r) => return self.err_near(
                    "cannot subtract non-numerics",
                    op, format!("{:?} - {:?}", l, r)),
            },
            Star => match (l, r) {
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

    fn visit_assignment(&mut self, _expr: &Expr, id: &Token, val: &Expr) -> Result<Object> {
        let v = val.accept(self)?;
        self.env.assign_at(id, v, self.locals.get(val))
    }

    fn visit_call(&mut self, _expr: &Expr, callee: &Expr, paren: &Token, args: &[Expr]) -> Result<Object> {
        match callee.accept(self)? {
            Object::Func(ref func) => self.dispatch_call(func, paren, args),
            Object::Class(ref cls) => self.dispatch_call(&Callable::init(cls), paren, args),
            x => self.err_near(
                "can only call functions and classes",
                paren, format!("{}", x)),
        }
    }

    fn visit_get(&mut self, _expr: &Expr, callee: &Expr, prop: &Token) -> Result<Object> {
        match callee.accept(self)? {
            Object::Instance(ref inst) => inst.get(prop),
            _ => Err(Error::Runtime(
                prop.line,
                "only instances have properties".to_owned(),
                prop.lexeme.to_owned(),
            ))
        }
    }

    fn visit_set(&mut self, _expr: &Expr, settee: &Expr, prop: &Token, val: &Expr) -> Result<Object> {
        if let Object::Instance(ref inst) = settee.accept(self)? {
            inst.set(prop, val.accept(self)?)
        } else {
            Err(Error::Runtime(
                prop.line,
                "only instances have fields".to_owned(),
                prop.lexeme.to_owned()))
        }
    }

    fn visit_this(&mut self, expr: &Expr, tkn: &Token) -> Result<Object> {
        self.lookup_var(tkn, expr)
    }

    fn visit_super(&mut self, expr: &Expr, tkn: &Token, method: &Token) -> Result<Object> {
        let dist: usize = *self.locals.get(expr)
            .expect("dist always available for super");

        let parent = match self.env.get_at(tkn, Some(&dist))? {
            Object::Class(ref c) => Rc::clone(c),
            _ => return Err(Error::Runtime(tkn.line,
                                           "unexpected super".to_owned(),
                                           tkn.lexeme.to_owned())),
        };

        let inst = match self.env.get_at(&THIS_ID, Some(&(dist - 1)))? {
            Object::Instance(ref i) => i.clone(),
            _ => return Err(Error::Runtime(tkn.line,
                                           "unexpected this".to_owned(),
                                           tkn.lexeme.to_owned())),
        };

        match parent.find_method(&method.lexeme) {
            Some(m) => Ok(Object::Func(m.bind(&inst))),
            None => Err(Error::Runtime(
                method.line,
                "undefined property".to_owned(),
                method.lexeme.to_owned())),
        }
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_empty(&mut self, _stmt: &Stmt) -> Result<()> { Ok(()) }

    fn visit_break(&mut self, _stmt: &Stmt, tkn: &Token) -> Result<()> {
        Err(Error::Break(tkn.line))
    }

    fn visit_expr_stmt(&mut self, stmt: &Stmt, expr: &Expr) -> Result<()> {
        if self.repl {
            self.visit_print(stmt, expr)
        } else {
            expr.accept(self).map(|_| ())
        }
    }

    fn visit_print(&mut self, _stmt: &Stmt, expr: &Expr) -> Result<()> {
        let obj = expr.accept(self)?;
        Writer::writeln(&self.stdout, &format!("{}", obj))
    }

    fn visit_decl(&mut self, _stmt: &Stmt, id: &Token, init: Option<&Expr>) -> Result<()> {
        let val: Object = init.map_or_else(
            || Ok(Object::Literal(Literal::Nil)),
            |e| e.accept(self))?;

        self.env.define(id, val)
    }

    fn visit_block(&mut self, _stmt: &Stmt, body: &[Stmt]) -> Result<()> {
        let mut scope = self.scoped();
        for stmt in body { stmt.accept(&mut scope)?; }
        Ok(())
    }

    fn visit_if(&mut self, _stmt: &Stmt, cond: &Expr, then: &Stmt, els: Option<&Stmt>) -> Result<()> {
        if cond.accept(self)?.is_truthy() {
            return then.accept(self);
        }

        if let Some(stmt) = els {
            return stmt.accept(self);
        }

        Ok(())
    }

    fn visit_while(&mut self, _stmt: &Stmt, cond: &Expr, body: &Stmt) -> Result<()> {
        while cond.accept(self)?.is_truthy() {
            match body.accept(self) {
                Err(Error::Break(_)) => return Ok(()),
                Err(e) => return Err(e),
                _ => (),
            };
        }
        Ok(())
    }

    fn visit_func(&mut self, _stmt: &Stmt, id: &Token, params: &[Token], body: Rc<Stmt>) -> Result<()> {
        let f = Callable::new(Env::from_weak(&self.env), params, &body, false);
        self.env.define(id, Object::Func(f))
    }

    fn visit_return(&mut self, _stmt: &Stmt, tkn: &Token, val: Option<&Expr>) -> Result<()> {
        let res = match val {
            Some(expr) => expr.accept(self)?,
            None => Object::Literal(Literal::Nil),
        };

        Err(Error::Return(tkn.line, res))
    }

    fn visit_class(&mut self, _stmt: &Stmt, id: &Token, parent: Option<&Expr>, methods: &[Stmt]) -> Result<()> {
        let env = Env::from_weak(&self.env);

        let superclass = if let Some(p) = parent {
            let c = match p.accept(self)? {
                Object::Class(ref c) => Rc::clone(c),
                _ => return Err(Error::Parse(id.line,
                                             "superclass must be a class".to_owned(),
                                             id.lexeme.to_owned())),
            };

            env.define(&SUPER_ID, Object::Class(Rc::clone(&c)))?;

            Some(c)
        } else { None };

        let mut ms = HashMap::with_capacity(methods.len());
        for method in methods {
            match *method {
                Stmt::Function(ref id, ref params, ref body) => {
                    let f = Callable::new(
                        Rc::clone(&env),
                        params,
                        body,
                        id.lexeme.eq(INITIALIZER_FUNC));

                    ms.insert(id.lexeme.clone(), f);
                }
                _ => unreachable!(),
            }
        };


        let cls = Rc::new(LoxClass::new(&id.lexeme, superclass, ms));
        self.env.define(id, Object::Class(cls))
    }
}

impl Interpreter {
    fn scoped(&self) -> Interpreter {
       let i = Interpreter {
            env: Env::from(&self.env),
            locals: Rc::clone(&self.locals),
            repl: false,
            stdout: Rc::clone(&self.stdout),
        };

        debug_create!("Interpreter::Scoped ({} parent refs now)", Rc::strong_count(&i.locals)-1);

        i
    }

    fn visit_logical(&mut self, lhs: &Expr, op: &Token, rhs: &Expr) -> Result<Object> {
        use ast::token::Type::{Or, And};
        use ast::token::Literal::Boolean;

        let l: Object = lhs.accept(self)?;

        let res: Literal = match op.typ {
            And if l.is_truthy() => Boolean(rhs.accept(self)?.is_truthy()),
            Or if l.is_truthy() => Boolean(true),
            Or => Boolean(rhs.accept(self)?.is_truthy()),
            _ => Boolean(false),
        };

        Ok(Object::Literal(res))
    }

    fn lookup_var(&mut self, id: &Token, expr: &Expr) -> Result<Object> {
        self.env.get_at(id, self.locals.get(expr))
    }

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

    fn dispatch_call(&mut self, callee: &Callable, paren: &Token, args: &[Expr]) -> Result<Object> {
        if callee.arity() != args.len() {
            return self.err_near(
                &format!("expected {} arguments but got {}", callee.arity(), args.len()),
                paren, "".to_string());
        }

        let mut params: Vec<Object> = Vec::with_capacity(args.len());
        for arg in args {
            params.push(arg.accept(self)?);
        }

        callee.call(self, &params, paren)
    }
}
