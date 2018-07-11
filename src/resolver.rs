use ast::expr::Visitor as ExprVisitor;
use ast::expr::Expr;
use ast::stmt::Visitor as StmtVisitor;
use ast::stmt::Stmt;
use result::{Result, Error};
use interpreter::Interpreter;
use std::collections::HashMap;
use functions::Type as FunctionType;
use ast::token::Token;
use std::rc::Rc;
use class::{THIS_ID, SUPER_ID};
use class::Type as ClassType;
use functions::INITIALIZER_FUNC;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    fn new(i: &'a mut Interpreter) -> Resolver {
        Self {
            interpreter: i,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(i: &'a mut Interpreter, stmt: &Stmt) -> Result<&'a mut Interpreter> {
        let mut res = Self::new(i);
        stmt.accept(&mut res)?;
        Ok(res.interpreter)
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        Err(Error::Parse(
            0,
            format!("{:?}", expr),
            "".to_owned(),
        ))
    }

    fn visit_identifier(&mut self, expr: &Expr, id: &Token) -> Result<()> {
        let own_init: bool = self.scopes.last()
            .and_then(|s| s.get(&id.lexeme))
            .map_or(false, |d| !*d);

        if own_init {
            return Err(Error::Parse(
                id.line,
                "cannot read local variable in its own initializer.".to_owned(),
                id.lexeme.clone()));
        }

        self.resolve_local(id, expr);
        Ok(())
    }

    fn visit_literal(&mut self, _expr: &Expr, _lit: &Token) -> Result<()> {
        Ok(())
    }

    fn visit_grouping(&mut self, _expr: &Expr, inside: &Expr) -> Result<()> {
        inside.accept(self)
    }

    fn visit_unary(&mut self, _expr: &Expr, _op: &Token, rhs: &Expr) -> Result<()> {
        rhs.accept(self)
    }

    fn visit_binary(&mut self, _expr: &Expr, lhs: &Expr, _op: &Token, rhs: &Expr) -> Result<()> {
        lhs.accept(self)?;
        rhs.accept(self)
    }

    fn visit_assignment(&mut self, expr: &Expr, id: &Token, val: &Expr) -> Result<()> {
        val.accept(self)?;
        self.resolve_local(id, expr);
        Ok(())
    }

    fn visit_call(&mut self, _expr: &Expr, callee: &Expr, _paren: &Token, args: &[Expr]) -> Result<()> {
        callee.accept(self)?;

        for arg in args {
            arg.accept(self)?;
        }

        Ok(())
    }

    fn visit_get(&mut self, _expr: &Expr, callee: &Expr, _prop: &Token) -> Result<()> {
        callee.accept(self)
    }

    fn visit_set(&mut self, _expr: &Expr, settee: &Expr, _prop: &Token, val: &Expr) -> Result<()> {
        val.accept(self)?;
        settee.accept(self)
    }

    fn visit_this(&mut self, expr: &Expr, tkn: &Token) -> Result<()> {
        if self.current_class == ClassType::None {
            return Err(Error::Parse(
                tkn.line,
                "cannot use 'this' outside of a class".to_owned(),
                tkn.lexeme.to_owned(),
            ));
        }

        self.resolve_local(tkn, expr);
        Ok(())
    }

    fn visit_super(&mut self, expr: &Expr, tkn: &Token, _method: &Token) -> Result<()> {
        match self.current_class {
            ClassType::None => Err(Error::Parse(
                tkn.line,
                "cannot use 'super' outside of a class".to_owned(),
                tkn.lexeme.to_owned())),
            ClassType::Class => Err(Error::Parse(
                tkn.line,
                "cannot use 'super' in a class with no superclass".to_owned(),
                tkn.lexeme.to_owned())),
            ClassType::SubClass => {
                self.resolve_local(tkn, expr);
                Ok(())
            }
        }
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_stmt(&mut self, _stmt: &Stmt) -> Result<()> { Ok(()) }

    fn visit_expr_stmt(&mut self, _stmt: &Stmt, expr: &Expr) -> Result<()> {
        expr.accept(self)
    }

    fn visit_print(&mut self, _stmt: &Stmt, expr: &Expr) -> Result<()> {
        expr.accept(self)
    }

    fn visit_decl(&mut self, _stmt: &Stmt, id: &Token, init: Option<&Expr>) -> Result<()> {
        self.declare(id)?;

        if let Some(expr) = init.as_ref() {
            expr.accept(self)?;
        }

        self.define(id)
    }

    fn visit_block(&mut self, _stmt: &Stmt, body: &[Stmt]) -> Result<()> {
        self.begin_scope();

        for s in body {
            s.accept(self)?;
        }

        self.end_scope();

        Ok(())
    }

    fn visit_if(&mut self, _stmt: &Stmt, cond: &Expr, then: &Stmt, els: Option<&Stmt>) -> Result<()> {
        cond.accept(self)?;
        then.accept(self)?;

        if let Some(stmt) = els {
            stmt.accept(self)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, _stmt: &Stmt, cond: &Expr, body: &Stmt) -> Result<()> {
        cond.accept(self)?;
        body.accept(self)
    }

    fn visit_func(&mut self, _stmt: &Stmt, id: &Token, params: &[Token], body: Rc<Stmt>) -> Result<()> {
        self.declare_and_define(id)?;
        self.resolve_function(params, body.as_ref(), FunctionType::Function)
    }

    fn visit_return(&mut self, _stmt: &Stmt, tkn: &Token, val: Option<&Expr>) -> Result<()> {
        use functions::Type::*;

        match self.current_function {
            None => return Err(Error::Parse(tkn.line,
                                            "cannot return from top-level code".to_owned(),
                                            tkn.lexeme.to_owned())),
            Initializer => return Err(Error::Parse(tkn.line,
                                                   "cannot return a value from an initializer".to_owned(),
                                                   tkn.lexeme.to_owned())),
            _ => ()
        };

        if let Some(expr) = val {
            expr.accept(self)?;
        }

        Ok(())
    }

    fn visit_class(&mut self, _stmt: &Stmt, id: &Token, parent: Option<&Expr>, methods: &[Stmt]) -> Result<()> {
        self.declare_and_define(id)?;
        let prev = self.current_class;
        self.current_class = ClassType::Class;

        if let Some(expr) = parent {
            self.current_class = ClassType::SubClass;
            expr.accept(self)?;
            self.begin_scope();
            self.declare_and_define(&SUPER_ID)?;
        }

        self.begin_scope();
        self.declare_and_define(&THIS_ID)?;

        for method in methods {
            match *method {
                Stmt::Function(ref id, ref params, ref body) => {
                    let typ = if id.lexeme.eq(INITIALIZER_FUNC) {
                        FunctionType::Initializer
                    } else { FunctionType::Method };

                    self.resolve_function(params, body.as_ref(), typ)?;
                }
                _ => unreachable!(),
            };
        }

        self.end_scope();
        if parent.is_some() { self.end_scope(); }
        self.current_class = prev;

        Ok(())
    }
}

impl<'a> Resolver<'a> {
    fn begin_scope(&mut self) { self.scopes.push(HashMap::new()); }

    fn end_scope(&mut self) { self.scopes.pop(); }

    fn declare(&mut self, id: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.insert(id.lexeme.to_owned(), false).is_some() {
                return Err(Error::Parse(
                    id.line,
                    "variable already defined with that name in this scope".to_owned(),
                    id.lexeme.to_owned()));
            }
        }

        Ok(())
    }

    fn define(&mut self, id: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(id.lexeme.to_owned(), true);
        }

        Ok(())
    }

    fn declare_and_define(&mut self, id: &Token) -> Result<()> {
        self.declare(id)?;
        self.define(id)
    }

    fn resolve_local(&mut self, id: &Token, expr: &Expr) {
        let l = self.scopes.len();
        for i in (0..l).rev() {
            if self.scopes[i].get(&id.lexeme).is_some() {
                self.interpreter.resolve(expr, l - 1 - i);
                return;
            }
        }
    }

    fn resolve_function(&mut self, params: &[Token], body: &Stmt, typ: FunctionType) -> Result<()> {
        let prev = self.current_function;
        self.current_function = typ;
        self.begin_scope();

        for param in params {
            self.declare_and_define(param)?;
        }

        body.accept(self)?;

        self.end_scope();
        self.current_function = prev;
        Ok(())
    }
}
