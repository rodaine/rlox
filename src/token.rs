use std::fmt;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Token {
    pub typ: Type,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u64,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {} {:?}", self.typ, self.lexeme, self.literal)
    }
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f64),
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Type {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    EOF,
}

lazy_static! {
    pub static ref RESERVED: HashMap<&'static str, Type> = [
        ("and", Type::And),
        ("class", Type::Class),
        ("else", Type::Else),
        ("false", Type::False),
        ("fun", Type::Fun),
        ("for", Type::For),
        ("if", Type::If),
        ("nil", Type::Nil),
        ("or", Type::Or),
        ("print", Type::Print),
        ("return", Type::Return),
        ("super", Type::Super),
        ("this", Type::This),
        ("true", Type::True),
        ("var", Type::Var),
        ("while", Type::While),
    ].iter().cloned().collect();
}
