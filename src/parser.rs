#![allow(dead_code)]

use Boxer;
use ast::Expr;
use result::{Result, Error};
use scanner::Scanner;
use token::{Type, Token};
use token::Type::*;
use std::iter::Peekable;
use std::error::Error as StdError;

pub struct Parser<'a> {
    src: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(s: Scanner<'a>) -> Self { Parser { src: s.peekable() } }
    pub fn parse(&mut self) -> Result<Expr> { self.expression() }
}

impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr> { self.equality() }

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
            return Ok(Expr::Unary(op?, self.unary()?.boxed()))
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if let Some(Ok(tkn)) = self.check_next(&[Nil, True, False, String, Number]) {
            return match tkn.typ {
                Nil | True | False | Number | String => Ok(Expr::Literal(tkn.literal.unwrap())),
                _ => Err(Parser::unexpected(&tkn)),
            }
        }

        if let Some(Ok(_)) = self.check_next(&[LeftParen]) {
            let expr = self.expression()?;
            let _ = self.must_next(&[RightParen])?;
            return Ok(Expr::Grouping(expr.boxed()))
        }

        Err(self.peek_err())
    }
}

impl<'a> Parser<'a> {
    fn check(&mut self, types: &[Type]) -> bool {
        match self.src.peek() {
            Some(&Ok(ref t)) => t.in_types(types.iter().clone().collect()),
            _ => false,
        }
    }

    fn check_next(&mut self, types: &[Type]) -> Option<Result<Token>> {
        if self.check(types) {
            return self.src.next()
        }
        None
    }

    fn must_next(&mut self, types: &[Type]) -> Result<Token> {
        if let Some(res) = self.check_next(types) {
            return res
        }

        Err(self.peek_err())
    }

    fn peek_err(&mut self) -> Box<StdError> {
        {
            // peek for EOF and unexpected tokens
            let pk: Option<&Result<Token>> = self.src.peek();

            if pk.is_none() {
                return Parser::eof()
            }

            if let Ok(tkn) = pk.unwrap().as_ref() {
                return Parser::unexpected(tkn)
            }
        }

        // lexical or other error encountered
        self.src.next().unwrap().unwrap_err()
    }

    #[cfg_attr(feature = "cargo-clippy", allow(while_let_on_iterator))]
    fn synchronize(&mut self) -> Result<()> {
        while let Some(tkn) = self.src.next() {
            if tkn?.typ == Semicolon && self.check(&[
                Class,
                Fun,
                Var,
                For,
                If,
                While,
                Print,
                Return,
            ]) {
                return Ok(())
            }
        }

        Ok(())
    }

    fn eof() -> Box<StdError> {
        Error::Parse(0, "".to_string(), "unexpected EOF".to_string()).boxed()
    }

    fn unexpected(tkn: &Token) -> Box<StdError> {
        Error::Parse(tkn.line, "unexpected token".to_string(), tkn.lexeme.clone()).boxed()
    }
}
