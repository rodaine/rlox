//! A module describing Lox tokens.

use std::fmt;
use std::collections::HashMap;

/// A Token read from source.
///
/// A Token describes the lexeme read from a source.
#[derive(Debug)]
pub struct Token {
    /// This token's type
    pub typ: Type,
    /// The raw lexeme read from source
    pub lexeme: String,
    /// The literal value for string and number types
    pub literal: Option<Literal>,
    /// The starting line number this token was read from
    pub line: u64,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {} {:?}", self.typ, self.lexeme, self.literal)
    }
}

/// Describes a literal string or number value
#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f64),
}

/// Describes the type of a Token
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
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

/// Returns a matching Token Type if a keyword is reserved
///
/// # Examples
///
/// ```
/// # extern crate rlox;
/// # use rlox::token::*;
/// # fn main() {
/// let t = reserved("true").expect("'true' is a reserved keyword");
/// assert_eq!(t, &Type::True);
///
/// assert!(reserved("foo").is_none());
/// # }
/// ```
pub fn reserved(keyword: &str) -> Option<&Type> {
    RESERVED.get(keyword)
}

lazy_static! {
    static ref RESERVED: HashMap<&'static str, Type> = [
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
