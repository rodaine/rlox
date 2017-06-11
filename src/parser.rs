use std::iter::Peekable;

use ast::expr::Expr;
use ast::stmt::Stmt;
use Boxer;
use result::{Result, Error};
use scanner::Scanner;
use token::{Type, Token};
use token::Type::*;


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
            return None
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
            _ => Err(Parser::unexpected(&tkn)),
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr: Expr = self.expression()?;
        self.must_next(&[Semicolon])?;
        Ok(Stmt::Print(expr))
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
}

// Private, expression-related methods on the Parser
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr> { self.assignment() }

    fn assignment(&mut self) -> Result<Expr> {
        let expr: Expr = self.equality()?;

        if let Some(res) = self.check_next(&[Equal]) {
            let eq: Token = res?;

            return match expr {
                Expr::Identifier(id) => Ok(Expr::Assignment(id, self.assignment()?.boxed())),
                _ => Err(Parser::unexpected(&eq)),
            }
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
            return Ok(Expr::Unary(op?, self.unary()?.boxed()))
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if let Some(Ok(tkn)) = self.check_next(&[Nil, True, False, String, Number, Identifier]) {
            return match tkn.typ {
                Identifier => Ok(Expr::Identifier(tkn.lexeme.clone())),
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

    fn peek_err(&mut self) -> Error {
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

    fn synchronize(&mut self) {
        loop {
            if let Some(&Err(_)) = self.src.peek() {
                return
            }

            let tkn: Option<Result<Token>> = self.src.next();

            if tkn.is_none() { return }

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
                    return
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
