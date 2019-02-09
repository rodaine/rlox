use crate::scanner::Scanner;
use std::rc::Rc;
use std::result;
use std::f64::NAN;

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

        while self.current.is_some() {
            self.declaration()
        }

        if self.has_error { Err(Error {}) } else { Ok(self.chunk) }
    }

    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.variable_declaration()
        } else {
            self.statement();
        }

        self.synchronize();
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn variable_declaration(&mut self) {
        let var = self.parse_variable();

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.write_simple(OpCode::Nil);
        }
        self.consume(TokenType::Semicolon);

        self.define_variable(var)
    }

    fn parse_variable(&mut self) -> usize {
        self.consume(TokenType::Identifier);
        return self.identifier_const();
    }

    fn define_variable(&mut self, idx: usize) {
        use self::OpCode::*;
        self.chunk.write_idx(self.prev_line(), &[DefineGlobal8, DefineGlobal16, DefineGlobal24], idx);
    }

    fn identifier_const(&mut self) -> usize {
        let id = self.previous.as_ref().unwrap();
        return self.chunk.make_const(id.lex().into());
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon);
        self.write_simple(OpCode::Print)
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon);
        self.write_simple(OpCode::Pop)
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();

        let can_assign = prec <= Precedence::Assignment;

        if !self.call_prefix(self.prev_type(), can_assign) {
            self.error("expect expression");
            return;
        }

        while prec <= Precedence::from(self.curr_type()) {
            self.advance();
            self.call_infix(self.prev_type());
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.error("invalid assignment target");
            self.expression();
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

    fn number(&mut self) {
        let val = self.previous.as_ref()
            .map_or(NAN, |t| t.lex().value().parse().unwrap_or(NAN));
        self.chunk.write_const(self.prev_line(), val.into());
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

    fn string(&mut self) {
        let mut lex = self.previous.as_ref().map(|t| t.lex().clone()).unwrap();
        // tim quotes
        lex.start += 1;
        lex.length -= 2;

        // TODO: translate escapes here!
        self.chunk.write_const(self.prev_line(), lex.into());
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign);
    }

    fn named_variable(&mut self, can_assign: bool) {
        use self::OpCode::*;
        let lex = self.previous.as_ref().unwrap().lex();
        let idx = self.chunk.make_const(lex.into());
        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            self.chunk.write_idx(self.prev_line(), &[SetGlobal8, SetGlobal16, SetGlobal24], idx);
        } else {
            self.chunk.write_idx(self.prev_line(), &[GetGlobal8, GetGlobal16, GetGlobal24], idx);
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

    fn check(&self, typ: TokenType) -> bool {
        self.current.as_ref().map_or(false, |t| t.typ() == typ)
    }

    fn matches(&mut self, typ: TokenType) -> bool {
        if !self.check(typ) {
            return false;
        }

        self.advance();
        return true;
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
        } else {
        }
    }

    fn call_prefix(&mut self, typ: TokenType, can_assign: bool) -> bool {
        use self::TokenType::*;
        match typ {
            LeftParen => self.grouping(),
            Minus | Bang => self.unary(),
            Number => self.number(),
            True | False | Nil => self.literal(),
            String => self.string(),
            Identifier => self.variable(can_assign),
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

    fn synchronize(&mut self) {
        if !self.has_error {
            return;
        }
        self.has_error = false;

        while self.current.is_some() {
            if self.previous.as_ref().unwrap().typ() == TokenType::Semicolon {
                return;
            }

            use self::TokenType::*;
            match self.current.as_ref().unwrap().typ() {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => (),
            };
        }

        self.advance();
    }
}
