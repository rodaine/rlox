//! A module describing Lox tokens.

use std::fmt;
use std::default;
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;

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

impl Token {
    pub fn in_types(&self, types: &[Type]) -> bool {
        let hs: HashSet<&Type> = types.iter().clone().collect();
        hs.contains(&self.typ)
    }
}

impl default::Default for Token {
    fn default() -> Self {
        Token {
            typ: Type::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: 0
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {} {:?}", self.typ, self.lexeme, self.literal)
    }
}

/// Describes a literal string or number value
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Literal {
    pub fn is_truthy(&self) -> bool {
        use token::Literal::*;

        match *self {
            Nil => false,
            Boolean(tf) => tf,
            Number(n) => n != 0.0,
            String(ref s) => !s.is_empty(),
        }
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use std::cmp::Ordering::*;
        use token::Literal::*;

        match (self, other) {
            (&Nil, &Nil) => Some(Equal),
            (&Number(l), &Number(r)) => l.partial_cmp(&r),
            (&String(ref l), &String(ref r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use token::Literal::*;

        match *self {
            Nil => write!(f, "nil"),
            Boolean(b) => write!(f, "{}", b),
            Number(n) => write!(f, "{}", n),
            String(ref s) => write!(f, "{}", s),
        }
    }
}

/// Describes the type of a Token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    Break,
    EOF,
}

impl Type {
    /// Returns a matching Token Type if a keyword is reserved
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rlox;
    /// # use rlox::token::*;
    /// # fn main() {
    /// let t = Type::reserved("true").expect("'true' is a reserved keyword");
    /// assert_eq!(t, &Type::True);
    ///
    /// assert!(Type::reserved("foo").is_none());
    /// # }
    /// ```
    pub fn reserved(keyword: &str) -> Option<&Self> {
        RESERVED.get(keyword)
    }
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
        ("break", Type::Break),
    ].iter().cloned().collect();
}
