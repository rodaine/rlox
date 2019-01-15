use crate::scanner::Scanner;
use std::rc::Rc;
use std::result;
use std::f64::NAN;

use crate::value::Value;
use crate::token::{Token, TokenType, ErrorType};
use crate::chunk::{Chunk, OpCode};

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn next(self) -> Self {
        use self::Precedence::*;
        match self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => Primary,
        }
    }
}

impl From<TokenType> for Precedence {
    fn from(typ: TokenType) -> Self {
        use self::TokenType::*;
        use self::Precedence::*;

        match typ {
            LeftParen | Dot => Call,
            Minus | Plus => Term,
            Slash | Star => Factor,
            BangEqual | EqualEqual => Equality,
            Greater | GreaterEqual | Less | LessEqual => Comparison,
            TokenType::And => Precedence::And,
            TokenType::Or => Precedence::Or,
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error();

pub type Result = result::Result<Chunk, Error>;

pub struct Compiler {
    scanner: Scanner,
    chunk: Chunk,
    previous: Option<Token>,
    current: Option<Token>,
    has_error: bool,
    panic_mode: bool,
}

impl Compiler {
    pub fn new(source: &Rc<String>, line: usize) -> Self {
        Self {
            scanner: Scanner::new(source, line),
            chunk: Chunk::default(),
            previous: None,
            current: None,
            has_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(mut self) -> Result {
        self.advance();

        self.expression();

        if self.current.is_some() {
            self.error("expected EOF");
            self.has_error = true;
        }

        self.write_simple(OpCode::Return);

        if self.has_error { Err(Error {}) } else { Ok(self.chunk) }
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();

        if !self.call_prefix(self.prev_type()) {
            self.error("expect expression");
            return;
        }

        while prec <= Precedence::from(self.curr_type()) {
            self.advance();
            self.call_infix(self.prev_type());
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen);
    }

    fn unary(&mut self) {
        let op = self.prev_type();
        self.parse_precedence(Precedence::Unary);
        match op {
            TokenType::Minus => self.write_simple(OpCode::Negate),
            TokenType::Bang => self.write_simple(OpCode::Not),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        use self::TokenType::*;

        let op = self.prev_type();
        self.parse_precedence(Precedence::from(op).next());
        match op {
            Plus => self.write_simple(OpCode::Add),
            Minus => self.write_simple(OpCode::Subtract),
            Star => self.write_simple(OpCode::Multiply),
            Slash => self.write_simple(OpCode::Divide),

            EqualEqual => self.write_simple(OpCode::Equal),
            BangEqual => self.write_simple2(OpCode::Equal, OpCode::Not),
            Greater => self.write_simple(OpCode::Greater),
            GreaterEqual => self.write_simple2(OpCode::Less, OpCode::Not),
            Less => self.write_simple(OpCode::Less),
            LessEqual => self.write_simple2(OpCode::Greater, OpCode::Not),

            _ => unreachable!(),
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let val = self.previous.as_ref()
            .map_or(NAN, |t| t.lex().value().parse().unwrap_or(NAN));
        self.chunk.write_const(self.prev_line(), Value::Number(val))
    }

    fn literal(&mut self) {
        use self::TokenType::*;
        match self.prev_type() {
            True => self.write_simple(OpCode::True),
            False => self.write_simple(OpCode::False),
            Nil => self.write_simple(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.take();

        loop {
            self.current = self.scanner.next();
            match self.current.as_ref().map(|t| t.typ()) {
                Some(TokenType::Error(_)) => self.error("syntax error"),
                _ => return,
            };
        }
    }

    fn consume(&mut self, typ: TokenType) {
        match &self.current {
            Some(tkn) if tkn.typ() == typ => self.advance(),
            _ => self.error(&format!("expected token {:?}", typ))
        }
    }

    fn write_simple(&mut self, op: OpCode) {
        self.chunk.write_simple(self.prev_line(), op)
    }

    pub fn write_simple2(&mut self, op1: OpCode, op2: OpCode) {
        self.write_simple(op1);
        self.write_simple(op2)
    }

    fn prev_type(&self) -> TokenType {
        Self::opt_typ(&self.previous)
    }

    fn prev_line(&self) -> usize {
        self.previous.as_ref().map_or(0, Token::ln)
    }

    fn curr_type(&self) -> TokenType {
        Self::opt_typ(&self.current)
    }

    fn opt_typ(tkn: &Option<Token>) -> TokenType {
        let typ = TokenType::Error(ErrorType::DoesNotExist);
        tkn.as_ref().map_or(typ, Token::typ)
    }

    fn error(&mut self, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.has_error = true;

        // TODO: pretty print this
        if let Some(t) = self.current.as_ref() {
            eprintln!("[Error] {}: {:?}", msg, t);
        } else {
            eprintln!("[Error] {}", msg)
        }
    }

    fn call_prefix(&mut self, typ: TokenType) -> bool {
        use self::TokenType::*;
        match typ {
            LeftParen => self.grouping(),
            Minus | Bang => self.unary(),
            Number => self.number(),
            True | False | Nil => self.literal(),
            _ => return false,
        };
        true
    }

    fn call_infix(&mut self, typ: TokenType) {
        use self::TokenType::*;
        match typ {
            Minus | Plus | Slash | Star |
            BangEqual | EqualEqual | Greater | GreaterEqual | Less | LessEqual => self.binary(),
            _ => {}
        }
    }
}
