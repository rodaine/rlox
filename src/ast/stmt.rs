use ast::token::Token;
use ast::expr::Expr;
use std::vec::Vec;
use std::rc::Rc;
use Boxer;

pub const FUNCTION_ARGS_MAX: usize = 8;

#[derive(Debug)]
pub enum Stmt {
    Empty,
    Break(Token),
    Expression(Expr),
    Print(Expr),
    Declaration(Token, Option<Box<Expr>>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Function(Token, Vec<Token>, Rc<Stmt>),
    Return(Token, Option<Box<Expr>>),
    Class(Token, Option<Box<Expr>>, Vec<Stmt>),
}

pub trait Visitor<T> {
    fn visit_stmt(&mut self, _stmt: &Stmt) -> T { unimplemented!() }

    fn visit_empty(&mut self, _stmt: &Stmt) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_break(&mut self, _stmt: &Stmt, _tkn: &Token) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_expr_stmt(&mut self, _stmt: &Stmt, _expr: &Expr) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_print(&mut self, _stmt: &Stmt, _expr: &Expr) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_decl(&mut self, _stmt: &Stmt, _id: &Token, _init: Option<&Expr>) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_block(&mut self, _stmt: &Stmt, _body: &[Stmt]) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_if(&mut self, _stmt: &Stmt, _cond: &Expr, _then: &Stmt, _els: Option<&Stmt>) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_while(&mut self, _stmt: &Stmt, _cond: &Expr, _body: &Stmt) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_func(&mut self, _stmt: &Stmt, _id: &Token, _params: &[Token], _body: Rc<Stmt>) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_return(&mut self, _stmt: &Stmt, _tkn: &Token, _val: Option<&Expr>) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_class(&mut self, _stmt: &Stmt, _id: &Token, _parent: Option<&Expr>, _methods: &[Stmt]) -> T {
        self.visit_stmt(_stmt)
    }
}

impl Stmt {
    pub fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        use ast::stmt::Stmt::*;
        match *self {
            Empty =>
                v.visit_empty(self),
            Break(ref tkn) =>
                v.visit_break(self, tkn),
            Expression(ref expr) =>
                v.visit_expr_stmt(self, expr),
            Print(ref expr) =>
                v.visit_print(self, expr),
            Declaration(ref id, ref init) =>
                v.visit_decl(self,
                             id,
                             init.as_ref().map(|e| e.as_ref())),
            Block(ref body) =>
                v.visit_block(self, body),
            If(ref cond, ref then, ref els) =>
                v.visit_if(self, cond, then.as_ref(), els.as_ref()
                    .map(|bs| bs.as_ref())),
            While(ref cond, ref body) =>
                v.visit_while(self, cond, body.as_ref()),
            Function(ref id, ref params, ref body) =>
                v.visit_func(self, id, params, Rc::clone(body)),
            Return(ref tkn, ref val) =>
                v.visit_return(self,
                               tkn,
                               val.as_ref().map(|e| e.as_ref())),
            Class(ref id, ref parent, ref methods) =>
                v.visit_class(self,
                              id,
                              parent.as_ref().map(|e| e.as_ref()),
                              methods),
        }
    }
}

impl Boxer for Stmt {}
