//! A module describing the Lox token scanner.

use std::str::Chars;
use token::{Token, Type, Literal, reserved};
use std::collections::{HashSet, VecDeque};
use std::ops::Index;
use Result;
use Error;

/// Scanner is an iterator that consumes a `Chars` iterator, returning `Result<Token>`.
///
/// Once an EOF token or Error has been returned, no more tokens will be emitted.
///
/// # Examples
/// ```
/// # extern crate rlox;
/// # use rlox::scanner::*;
/// # use rlox::token;
/// # fn main() {
/// let code = "num = 123";
/// let mut scanner = Scanner::new(code.chars());
///
/// let ident = scanner.next().expect("should have token").unwrap();
/// assert_eq!(token::Type::Identifier, ident.typ);
/// assert_eq!("num", ident.lexeme);
///
/// let eq = scanner.next().expect("should have token").unwrap();
/// assert_eq!(token::Type::Equal, eq.typ);
///
/// let lit = scanner.next().expect("should have token").unwrap();
/// assert_eq!(token::Type::Number, lit.typ);
/// assert_eq!(token::Literal::Number(123.), lit.literal.expect("should have a literal"));
///
/// let eof = scanner.next().expect("should have token").unwrap();
/// assert_eq!(token::Type::EOF, eof.typ);
///
/// assert!(scanner.next().is_none());
/// # }
/// ```
pub struct Scanner<'a> {
    src: Chars<'a>,
    peeks: VecDeque<char>,
    lexeme: String,
    line: u64,
    eof: bool,
}

impl<'a> Scanner<'a> {
    /// Creates a new Scanner off a Chars iterator.
    pub fn new(c: Chars<'a>) -> Self {
        Scanner {
            src: c,
            peeks: VecDeque::with_capacity(2),
            lexeme: "".to_string(),
            line: 1,
            eof: false,
        }
    }
}

impl<'a> Scanner<'a> {
    fn advance(&mut self) -> Option<char> {
        if self.eof {
            return None
        }

        match self.peeks.len() {
            0 => self.src.next(),
            _ => self.peeks.pop_front(),
        }.or_else(|| {
            self.eof = true;
            Some('\0')
        }).and_then(|c| {
            self.lexeme.push(c);
            Some(c)
        })
    }

    fn lookahead(&mut self, n: usize) -> char {
        assert!(n > 0, "lookahead must be greater than zero");

        while self.peeks.len() < n {
            self.src.next().
                or(Some('\0')).
                map(|c| self.peeks.push_back(c));
        }

        *self.peeks.index(n - 1)
    }

    fn peek(&mut self) -> char {
        self.lookahead(1)
    }

    fn peek_next(&mut self) -> char {
        self.lookahead(2)
    }

    fn match_advance(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance().unwrap();
            return true
        }

        false
    }

    fn advance_until(&mut self, c: HashSet<char>) -> char {
        let mut last = '\0';

        loop {
            match self.peek() {
                ch if c.contains(&ch) || ch == '\0' => break,
                ch => {
                    last = ch;
                    self.advance()
                }
            };
        };
        last
    }
}

impl<'a> Scanner<'a> {
    fn static_token(&self, typ: Type) -> Option<Result<Token>> {
        self.literal_token(typ, None)
    }

    fn literal_token(&self, typ: Type, lit: Option<Literal>) -> Option<Result<Token>> {
        Some(Ok(Token {
            typ: typ,
            literal: lit,
            line: self.line,
            lexeme: self.lexeme.clone(),
        }))
    }

    fn err(&self, msg: &str) -> Option<Result<Token>> {
        Some(Err(Error::Lexical(self.line, msg.to_string(), self.lexeme.clone()).boxed()))
    }

    fn match_static_token(&mut self, c: char, m: Type, u: Type) -> Option<Result<Token>> {
        if self.match_advance(c) {
            self.static_token(m)
        } else {
            self.static_token(u)
        }
    }

    fn string(&mut self) -> Option<Result<Token>> {
        loop {
            let last = self.advance_until(['\n', '"'].iter().cloned().collect());

            match self.peek() {
                '\n' => self.line += 1,
                '"' if last == '\\' => { self.lexeme.pop(); }
                '"' => break,
                '\0' => return self.err("unterminated string"),
                _ => return self.err("unexpected character"),
            };

            self.advance();
        }

        self.advance();

        let lit: String = self.lexeme.clone()
            .chars()
            .skip(1)
            .take(self.lexeme.len() - 2)
            .collect();

        self.literal_token(Type::String, Some(Literal::String(lit)))
    }

    fn number(&mut self) -> Option<Result<Token>> {
        while self.peek().is_digit(10) { self.advance(); };

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
            while self.peek().is_digit(10) { self.advance(); };
        }

        if let Ok(lit) = self.lexeme.clone().parse::<f64>() {
            return self.literal_token(Type::Number, Some(Literal::Number(lit)));
        }

        self.err("invalid numeric")
    }

    fn identifier(&mut self) -> Option<Result<Token>> {
        while is_alphanumeric(self.peek()) { self.advance(); }

        let lex: &str = self.lexeme.as_ref();
        let typ = reserved(lex)
            .map_or(Type::Identifier, |t| t.clone());

        match typ {
            Type::Nil => self.literal_token(typ, Some(Literal::Nil)),
            Type::True => self.literal_token(typ, Some(Literal::Boolean(true))),
            Type::False => self.literal_token(typ, Some(Literal::Boolean(false))),
            _ => self.static_token(typ)
        }
    }

    fn line_comment(&mut self) {
        self.advance_until(['\n'].iter().cloned().collect());
        self.lexeme.clear();
    }

    fn block_comment(&mut self) {
        self.advance(); // *

        loop {
            let last = self.advance_until(['\n', '/'].iter().cloned().collect());
            let next = self.peek();
            match (last, next) {
                (_, '\n') => self.line += 1,
                ('*', '/') => {
                    self.advance(); // *
                    self.advance(); // /
                    break;
                }
                (_, '\0') => break,
                (_, _) => (), // noop
            }
            self.advance();
        }

        self.lexeme.clear();
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        use token::Type::*;

        if self.eof {
            return None
        }

        self.lexeme.clear();

        loop {
            match self.advance().unwrap() {
                '\0' => {
                    self.eof = true;
                    return self.static_token(EOF)
                }

                '(' => return self.static_token(LeftParen),
                ')' => return self.static_token(RightParen),
                '{' => return self.static_token(LeftBrace),
                '}' => return self.static_token(RightBrace),
                ',' => return self.static_token(Comma),
                '.' => return self.static_token(Dot),
                '-' => return self.static_token(Minus),
                '+' => return self.static_token(Plus),
                ';' => return self.static_token(Semicolon),
                '*' => return self.static_token(Star),

                '!' => return self.match_static_token('=', BangEqual, Bang),
                '=' => return self.match_static_token('=', EqualEqual, Equal),
                '<' => return self.match_static_token('=', LessEqual, Less),
                '>' => return self.match_static_token('=', GreaterEqual, Greater),

                '"' => return self.string(),

                '/' => match self.peek() {
                    '/' => self.line_comment(),
                    '*' => self.block_comment(),
                    _ => return self.static_token(Slash),
                },

                c if c.is_whitespace() => {
                    self.lexeme.clear();
                    if c == '\n' {
                        self.line += 1
                    }
                }

                c if c.is_digit(10) => return self.number(),
                c if is_alphanumeric(c) => return self.identifier(),

                _ => return self.err("unexpected character"),
            }
        }
    }
}

/// Describes a type that can be converted into a token Scanner.
pub trait TokenIterator<'a> {
    fn tokens(self) -> Scanner<'a>;
}

impl<'a> TokenIterator<'a> for Chars<'a> {
    fn tokens(self) -> Scanner<'a> {
        Scanner::new(self)
    }
}

fn is_alphanumeric(c: char) -> bool {
    c.is_digit(36) || c == '_'
}
