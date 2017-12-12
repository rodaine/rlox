use std::iter::Peekable;

use ast::expr::Expr;
use ast::stmt::Stmt;
use Boxer;
use result::{Result, Error};
use scanner::Scanner;
use token::{Type, Token, Literal};
use token::Type::*;
use std::string::String as stdString;
use std::rc::Rc;

pub struct Parser<'a> {
    src: Peekable<Scanner<'a>>,
}

// Public methods on Parser
impl<'a> Parser<'a> {
    pub fn new(s: Scanner<'a>) -> Self { Parser { src: s.peekable() } }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.src.peek().is_none() || self.check_next(&[Type::EOF]).is_some() {
            return None;
        }

        let res = self.statement();
        if res.is_err() { self.synchronize(); }

        Some(res)
    }
}

// Private, statement-related methods on the Parser
impl<'a> Parser<'a> {
    fn statement(&mut self) -> Result<Stmt> {
        let n: Option<Result<Token>> = self.check_next(&[
            Semicolon,
            Print,
            Var,
            LeftBrace,
            If,
            While,
            For,
            Break,
            Fun,
            Return,
        ]);

        if n.is_none() {
            return self.expr_statement();
        }

        let tkn: Token = n.unwrap()?;

        match tkn.typ {
            Semicolon => Ok(Stmt::Empty),
            Print => self.print_statement(),
            Var => self.decl_statement(),
            LeftBrace => self.block_statement(),
            If => self.if_statement(),
            While => self.while_statement(),
            For => self.for_statement(),
            Break => self.break_statement(),
            Fun => self.function(),
            Return => self.return_statement(),
            _ => unreachable!(),
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr: Expr = self.expression()?;
        self.must_next(&[Semicolon])?;
        Ok(Stmt::Print(expr))
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.must_next(&[LeftParen])?;
        let expr: Expr = self.expression()?;
        self.must_next(&[RightParen])?;

        let then_stmt: Box<Stmt> = self.statement()?.boxed();

        match self.check_next(&[Else]) {
            Some(Err(e)) => Err(e),
            Some(Ok(_)) => Ok(Stmt::If(expr, then_stmt, Some(self.statement()?.boxed()))),
            None => Ok(Stmt::If(expr, then_stmt, None)),
        }
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        let expr: Expr = self.expression()?;
        let body: Box<Stmt> = self.statement()?.boxed();
        Ok(Stmt::While(expr, body))
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.must_next(&[LeftParen])?;

        let init: Option<Stmt> = match self.check_next(&[Semicolon, Var]) {
            None => Some(self.expr_statement()?),
            Some(t) => match t?.typ {
                Var => Some(self.decl_statement()?),
                Semicolon => None,
                _ => unreachable!(),
            }
        };

        let cond: Expr = match self.check_next(&[Semicolon]) {
            None => {
                let expr = self.expression()?;
                self.must_next(&[Semicolon])?;
                expr
            }
            Some(t) => {
                Expr::Literal(Token {
                    typ: True,
                    lexeme: "true".to_owned(),
                    literal: Some(Literal::Boolean(true)),
                    ..t?
                })
            }
        };

        let inc: Option<Stmt> = if self.check(&[RightParen]) {
            None
        } else {
            Some(Stmt::Expression(self.expression()?))
        };
        self.must_next(&[RightParen])?;

        let mut body: Stmt = self.statement()?;

        if inc.is_some() {
            body = Stmt::Block(vec![body, inc.unwrap()]);
        }

        body = Stmt::While(cond, body.boxed());

        if init.is_some() {
            body = Stmt::Block(vec![init.unwrap(), body])
        }

        Ok(body)
    }

    fn break_statement(&mut self) -> Result<Stmt> {
        let t: Token = self.must_next(&[Semicolon])?;
        Ok(Stmt::Break(t.line))
    }

    fn expr_statement(&mut self) -> Result<Stmt> {
        let expr: Expr = self.expression()?;
        self.must_next(&[Semicolon])?;
        Ok(Stmt::Expression(expr))
    }

    fn decl_statement(&mut self) -> Result<Stmt> {
        let id: Token = self.must_next(&[Identifier])?;

        if self.check_next(&[Equal]).is_none() {
            return Ok(Stmt::Declaration(id.lexeme, None));
        }

        let expr: Expr = self.expression()?;

        self.must_next(&[Semicolon])?;

        Ok(Stmt::Declaration(id.lexeme, Some(expr)))
    }

    fn block_statement(&mut self) -> Result<Stmt> {
        let mut stmts: Vec<Stmt> = Vec::new();

        while self.check_next(&[RightBrace]).is_none() && !self.src.peek().is_none() {
            stmts.push(self.statement()?);
        }

        Ok(Stmt::Block(stmts))
    }

    fn function(&mut self) -> Result<Stmt> {
        let name: Token = self.must_next(&[Identifier])?;
        self.must_next(&[LeftParen])?;

        let mut params: Vec<stdString> = Vec::new();

        if !self.check(&[RightParen]) {
            loop {
                if params.len() >= 8 {
                    return Err(Error::Parse(name.line,
                                            "cannot have more than 8 arguments".to_string(),
                                            name.lexeme));
                }

                params.push(self.must_next(&[Identifier])?.lexeme);

                if self.check_next(&[Comma]).is_none() {
                    break;
                }
            }
        }

        self.must_next(&[RightParen])?;
        self.must_next(&[LeftBrace])?;

        Ok(Stmt::Function(name.lexeme, params, Rc::new(self.block_statement()?)))
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let ln: u64 = match self.src.peek() {
            Some(res) => res.as_ref().map(|t| t.line).unwrap_or(0),
            None => 0,
        };

        let expr: Expr = if self.check(&[Semicolon]) {
            Expr::Literal(Token {
                typ: Nil,
                lexeme: "nil".to_owned(),
                ..Token::default()
            })
        } else {
            self.expression()?
        };

        self.must_next(&[Semicolon])?;

        Ok(Stmt::Return(ln, expr))
    }
}

// Private, expression-related methods on the Parser
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr> { self.assignment() }

    fn assignment(&mut self) -> Result<Expr> {
        let expr: Expr = self.logical_or()?;

        if let Some(res) = self.check_next(&[Equal]) {
            let eq: Token = res?;

            return match expr {
                Expr::Identifier(tkn) => Ok(Expr::Assignment(tkn, self.assignment()?.boxed())),
                _ => Err(Parser::unexpected(&eq)),
            };
        }

        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.logical_and()?;

        while let Some(op) = self.check_next(&[Or]) {
            expr = Expr::Binary(expr.boxed(), op?, self.logical_and()?.boxed());
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.equality()?;

        while let Some(op) = self.check_next(&[And]) {
            expr = Expr::Binary(expr.boxed(), op?, self.equality()?.boxed());
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.comparison()?;

        while let Some(op) = self.check_next(&[BangEqual, EqualEqual]) {
            expr = Expr::Binary(expr.boxed(), op?, self.comparison()?.boxed());
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.term()?;

        while let Some(op) = self.check_next(&[Greater, GreaterEqual, Less, LessEqual]) {
            expr = Expr::Binary(expr.boxed(), op?, self.term()?.boxed());
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.factor()?;

        while let Some(op) = self.check_next(&[Minus, Plus]) {
            expr = Expr::Binary(expr.boxed(), op?, self.factor()?.boxed());
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.unary()?;

        while let Some(op) = self.check_next(&[Star, Slash]) {
            expr = Expr::Binary(expr.boxed(), op?, self.unary()?.boxed());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Some(op) = self.check_next(&[Bang, Minus]) {
            return Ok(Expr::Unary(op?, self.unary()?.boxed()));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            expr = match self.check_next(&[LeftParen]) {
                Some(Err(e)) => return Err(e),
                Some(Ok(_)) => self.finish_call(expr)?,
                _ => break,
            };
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut args: Vec<Expr> = Vec::new();

        if !self.check(&[RightParen]) {
            loop {
                if args.len() >= 8 {
                    return Err(Error::Parse(0,
                                            "cannot have more than 8 arguments".to_string(),
                                            "".to_string()));
                }

                args.push(self.expression()?);

                match self.check_next(&[Comma]) {
                    Some(r) => r?,
                    None => break,
                };
            }
        }

        Ok(Expr::Call(
            callee.boxed(),
            self.must_next(&[RightParen])?,
            args))
    }

    fn primary(&mut self) -> Result<Expr> {
        if let Some(Ok(tkn)) = self.check_next(&[Nil, True, False, String, Number, Identifier]) {
            return match tkn.typ {
                Identifier => Ok(Expr::Identifier(tkn)),
                Nil | True | False | Number | String => Ok(Expr::Literal(tkn)),
                _ => Err(Parser::unexpected(&tkn)),
            };
        }

        if let Some(Ok(_)) = self.check_next(&[LeftParen]) {
            let expr = self.expression()?;
            let _ = self.must_next(&[RightParen])?;
            return Ok(Expr::Grouping(expr.boxed()));
        }

        Err(self.peek_err())
    }
}

// Token iterator related methods on the Parser
impl<'a> Parser<'a> {
    fn check(&mut self, types: &[Type]) -> bool {
        match self.src.peek() {
            Some(&Ok(ref t)) => t.in_types(types),
            _ => false,
        }
    }

    fn check_next(&mut self, types: &[Type]) -> Option<Result<Token>> {
        if self.check(types) {
            return self.src.next();
        }
        None
    }

    fn must_next(&mut self, types: &[Type]) -> Result<Token> {
        if let Some(res) = self.check_next(types) {
            return res;
        }

        Err(self.peek_err())
    }

    fn peek_err(&mut self) -> Error {
        {
            // peek for EOF and unexpected tokens
            let pk: Option<&Result<Token>> = self.src.peek();

            if pk.is_none() {
                return Parser::eof();
            }

            if let Ok(tkn) = pk.unwrap().as_ref() {
                return Parser::unexpected(tkn);
            }
        }

        // lexical or other error encountered
        self.src.next().unwrap().unwrap_err()
    }

    fn synchronize(&mut self) {
        loop {
            if let Some(&Err(_)) = self.src.peek() {
                return;
            }

            let tkn: Option<Result<Token>> = self.src.next();

            if tkn.is_none() { return; }

            if let Some(Ok(t)) = tkn {
                if t.typ == Semicolon && self.check(&[
                    Class,
                    Fun,
                    Var,
                    For,
                    If,
                    While,
                    Print,
                    Return,
                ]) {
                    return;
                }
            }
        }
    }

    fn eof() -> Error {
        Error::Parse(0, "".to_string(), "unexpected EOF".to_string())
    }

    fn unexpected(tkn: &Token) -> Error {
        let lex = match tkn.typ {
            EOF => "EOF".to_string(),
            _ => tkn.lexeme.clone(),
        };

        Error::Parse(tkn.line, "unexpected token".to_string(), lex)
    }
}

/// Describes a type that can be converted into a Parser.
pub trait StmtIterator<'a> {
    fn statements(self) -> Parser<'a>;
}

impl<'a> StmtIterator<'a> for Scanner<'a> {
    fn statements(self) -> Parser<'a> {
        Parser::new(self)
    }
}
