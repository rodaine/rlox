use std::rc::Rc;
use std::fmt;
use std::cmp::{PartialEq, Eq};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct InnerToken {
    pub line: usize,
    pub col_offset: usize,
    pub lex: Lexeme,
}

impl InnerToken {
    pub fn new(line: usize, lex: Lexeme) -> Self {
        Self { col_offset: lex.start, line, lex }
    }

    pub(crate) fn at_end(&self) -> bool { self.lex.at_end() }

    pub(crate) fn incr_line(&mut self) {
        self.line += 1;
        self.col_offset = self.lex.end();
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    t: TokenType,
    inner: InnerToken,
}

impl Token {
    pub fn new(typ: TokenType, inner: InnerToken) -> Self {
        Self { t: typ, inner }
    }

    pub fn typ(&self) -> TokenType { self.t }

    pub fn col(&self) -> usize { self.inner.lex.start - self.inner.col_offset + 1 }

    pub fn ln(&self) -> usize { self.inner.line }

    pub fn lex(&self) -> &Lexeme { &self.inner.lex }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    UnexpectedChar,
    UnterminatedString,
    DoesNotExist,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Single Char
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

    // Multi Char
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
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

    Error(ErrorType),
}

impl TokenType {
    pub fn reserved(ident: &str) -> Option<TokenType> {
        use self::TokenType::*;

        match ident {
            "and" => Some(And),
            "class" => Some(Class),
            "else" => Some(Else),
            "false" => Some(False),
            "for" => Some(For),
            "fun" => Some(Fun),
            "if" => Some(If),
            "nil" => Some(Nil),
            "or" => Some(Or),
            "print" => Some(Print),
            "return" => Some(Return),
            "super" => Some(Super),
            "this" => Some(This),
            "true" => Some(True),
            "var" => Some(Var),
            "while" => Some(While),
            "break" => Some(Break),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Lexeme {
    pub(crate) source: Rc<String>,
    pub(crate) start: usize,
    pub(crate) length: usize,
}

impl Lexeme {
    pub fn new(source: &Rc<String>, start: usize, length: usize) -> Self {
        Self {
            source: Rc::clone(source),
            start,
            length,
        }
    }

    pub fn from_str(src: String) -> Self {
        Self {
            length: src.len(),
            source: Rc::new(src),
            start: 0,
        }
    }

    pub fn value(&self) -> &str {
        &self.source[self.start..self.end()]
    }

    fn end(&self) -> usize {
        self.start + self.length
    }

    pub(crate) fn at_end(&self) -> bool {
        self.end() >= self.source.len()
    }

    pub(crate) fn char_at(&self, i: usize) -> char {
        if i >= self.source.len() {
            return '\0';
        }
        self.source[i..].chars().next().unwrap()
    }

    pub(crate) fn peek(&self) -> char {
        self.char_at(self.end())
    }

    pub(crate) fn peek_next(&self) -> char {
        self.char_at(self.end() + 1)
    }

    pub(crate) fn adv(&mut self) { self.length += 1; }

    pub(crate) fn adv_char(&mut self) -> char {
        let c = self.peek();
        self.adv();
        c
    }

    pub(crate) fn matches(&mut self, c: char) -> bool {
        if !self.at_end() && c == self.peek() {
            self.adv();
            return true;
        }

        return false;
    }

    pub(crate) fn shift(&mut self) {
        self.start += self.length;
        self.length = 0;
    }
}

impl fmt::Debug for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq for Lexeme {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for Lexeme {}

impl Hash for Lexeme {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value().hash(state)
    }
}
