use ast::expr::Visitor as ExprVisitor;
use ast::expr::Expr;
use ast::stmt::Visitor as StmtVisitor;
use ast::stmt::Stmt;
use result::{Result, Error};
use interpreter::Interpreter;
use std::collections::HashMap;
use functions::Type as FunctionType;
use ast::token::Token;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    fn new(i: &'a mut Interpreter) -> Resolver {
        Self {
            interpreter: i,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve(i: &'a mut Interpreter, stmt: &Stmt) -> Result<&'a mut Interpreter> {
        let mut res = Self::new(i);
        stmt.accept(&mut res)?;
        Ok(res.interpreter)
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_expr(&mut self, e: &Expr) -> Result<()> {
        match *e {
            Expr::Identifier(ref tkn) => self.visit_identifier(e, tkn),
            Expr::Assignment(ref tkn, ref val) => self.visit_assignment(tkn, val.as_ref()),
            Expr::Binary(ref lhs, _, ref rhs) => self.visit_binary(lhs.as_ref(), rhs.as_ref()),
            Expr::Call(ref callee, _, ref args) => self.visit_call(callee.as_ref(), args),
            Expr::Unary(_, ref e)
            | Expr::Grouping(ref e) => e.accept(self),
            _ => Ok(())
        }
    }
}

impl<'a> Resolver<'a> {
    fn resolve_local(&mut self, e: &Expr, name: &str) {
        let l = self.scopes.len();
        for i in (0..l).rev() {
            if self.scopes[i].get(name).is_some() {
                self.interpreter.resolve(e, l - 1 - i);
                return;
            }
        }
    }

    fn visit_identifier(&mut self, e: &Expr, tkn: &Token) -> Result<()> {
        let own_init: bool = self.scopes.last()
            .and_then(|s| s.get(&tkn.lexeme))
            .map_or(false, |d| !*d);

        if own_init {
            return Err(Error::Parse(
                0,
                "cannot read local variable in its own initializer.".to_owned(),
                tkn.lexeme.clone()));
        }

        self.resolve_local(e, &tkn.lexeme);
        Ok(())
    }

    fn visit_assignment(&mut self, tkn: &Token, val: &Expr) -> Result<()> {
        val.accept(self)?;
        self.resolve_local(val, &tkn.lexeme);
        Ok(())
    }

    fn visit_binary(&mut self, lhs: &Expr, rhs: &Expr) -> Result<()> {
        lhs.accept(self)?;
        rhs.accept(self)
    }

    fn visit_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<()> {
        callee.accept(self)?;

        for arg in args {
            arg.accept(self)?;
        }

        Ok(())
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_stmt(&mut self, s: &Stmt) -> Result<()> {
        match *s {
            Stmt::Block(ref stmts) => self.visit_block_stmt(stmts),
            Stmt::Declaration(ref name, ref init) => self.visit_decl(name, init),
            Stmt::Function(ref name, ref args, ref body) =>
                self.visit_function(name, args, body.as_ref()),
            Stmt::If(ref cond, ref then, ref els) =>
                self.visit_if(cond, then.as_ref(), els),
            Stmt::While(ref cond, ref body) => self.visit_while(cond, body.as_ref()),
            Stmt::Expression(ref expr)
            | Stmt::Print(ref expr) => expr.accept(self),
            Stmt::Return(_, ref expr) => self.visit_return(expr),
            _ => Ok(()),
        }
    }
}

impl<'a> Resolver<'a> {
    fn begin_scope(&mut self) { self.scopes.push(HashMap::new()); }
    fn end_scope(&mut self) { self.scopes.pop(); }

    fn declare(&mut self, name: &str) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.insert(name.to_owned(), false).is_some() {
                return Err(Error::Parse(
                    0,
                    "variable already defined with that name in this scope".to_owned(),
                    name.to_owned()));
            }
        }

        Ok(())
    }

    fn define(&mut self, name: &str) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_owned(), true);
        }

        Ok(())
    }

    fn declare_and_define(&mut self, name: &str) -> Result<()> {
        self.declare(name)?;
        self.define(name)
    }

    fn resolve_function(&mut self, args: &[String], body: &Stmt, typ: FunctionType) -> Result<()> {
        let prev = self.current_function;
        self.current_function = typ;
        self.begin_scope();

        for arg in args {
            self.declare_and_define(arg)?;
        }

        body.accept(self)?;

        self.end_scope();
        self.current_function = prev;
        Ok(())
    }

    fn visit_block_stmt(&mut self, stmts: &[Stmt]) -> Result<()> {
        self.begin_scope();

        for s in stmts {
            s.accept(self)?;
        }

        self.end_scope();

        Ok(())
    }

    fn visit_decl(&mut self, name: &str, initializer: &Option<Expr>) -> Result<()> {
        self.declare(name)?;

        if let Some(expr) = initializer.as_ref() {
            expr.accept(self)?;
        }

        self.define(name)?;

        Ok(())
    }

    fn visit_function(&mut self, name: &str, args: &[String], body: &Stmt) -> Result<()> {
        self.declare_and_define(name)?;
        self.resolve_function(args, body, FunctionType::Function)
    }

    fn visit_if(&mut self, cond: &Expr, then: &Stmt, els: &Option<Box<Stmt>>) -> Result<()> {
        cond.accept(self)?;
        then.accept(self)?;

        if els.is_some() {
            els.as_ref().unwrap().accept(self)?
        }

        Ok(())
    }

    fn visit_while(&mut self, cond: &Expr, body: &Stmt) -> Result<()> {
        cond.accept(self)?;
        body.accept(self)
    }

    fn visit_return(&mut self, expr: &Expr) -> Result<()> {
        if self.current_function == FunctionType::None {
            return Err(Error::Parse(0,
                                    "cannot return from top-level code".to_owned(),
                                    "".to_owned()));
        }

        expr.accept(self)
    }
}
