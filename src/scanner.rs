use std::str::Chars;
use std::iter::Peekable;
use result::{Result, Error};
use token::{Token, Type};

pub struct Scanner<'a> {
    src: Peekable<Chars<'a>>,
    lexeme: String,
    line: u64,
    eof: bool,
}

fn new(c: Chars) -> Scanner {
    Scanner {
        src: c.peekable(),
        lexeme: "".to_string(),
        line: 1,
        eof: false,
    }
}

impl<'a> Scanner<'a> {
    fn static_token(&self, typ: Type) -> Option<Result<Token>> {
        self.literal_token(typ, None)
    }

    fn literal_token(&self, typ: Type, lit: Option<()>) -> Option<Result<Token>> {
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
        match self.match_advance(c) {
            true => self.static_token(m),
            false => self.static_token(u),
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.eof {
            return None
        }

        self.src.next().or_else(|| {
            self.eof = true;
            Some('\0')
        }).and_then(|c| {
            self.lexeme.push(c);
            Some(c)
        })
    }

    fn peek(&mut self) -> char {
        self.src.peek()
            .map_or(Some('\0'), |ch| Some(*ch))
            .unwrap()
    }

    fn match_advance(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance().unwrap();
            return true
        }

        false
    }

    fn advance_until(&mut self, c: char) {
        loop {
            match self.peek() {
                c | '\0' => return,
                _ => self.advance()
            }
        }
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

        let c = self.advance().unwrap();

        match c {
            '(' => self.static_token(LeftParen),
            ')' => self.static_token(RightParen),
            '{' => self.static_token(LeftBrace),
            '}' => self.static_token(RightBrace),
            ',' => self.static_token(Comma),
            '.' => self.static_token(Dot),
            '-' => self.static_token(Minus),
            '+' => self.static_token(Plus),
            ';' => self.static_token(Semicolon),
            '*' => self.static_token(Star),
            '\0' => self.static_token(EOF),

            '!' => self.match_static_token('=', BangEqual, Bang),
            '=' => self.match_static_token('=', EqualEqual, Equal),
            '<' => self.match_static_token('=', LessEqual, Less),
            '>' => self.match_static_token('=', GreaterEqual, Greater),

            '/' => {
                if self.match_advance('/') {
                    self.advance_until('\n');
                    return self.next()
                }
                self.static_token(Slash)
            },



            _ => self.err("unexpected character"),
        }
    }
}

pub trait TokenIterator<'a> {
    fn tokens(self) -> Scanner<'a>;
}

impl<'a> TokenIterator<'a> for Chars<'a> {
    fn tokens(self) -> Scanner<'a> {
        new(self)
    }
}

//pub trait ToTokenIterator: Sized {
//    fn tokens(self) -> Scanner;
//}
//
//impl ToTokenIterator for Chars {
//    fn tokens(self) -> Scanner{
//        new(self)
//    }
//}

//struct Scanner {
//    source:
//}
//
//pub fn scan(source : &str) -> Result<Vec<Token>> {
//    let src: Vec<char> = source.chars().collect();
//    let s : String = src.into_iter().collect();
//    Err(Error::Usage.boxed())
//}
//
//fun scan_token()

//
//pub struct Scanner {
//    source: &'a str,
//    line:   u64,
//}
//
//impl Scanner {
//    pub fn new(source: &str) -> Scanner {
//        Scanner {
//            source: source,
//            line: 1,
//        }
//    }
//
//    pub fn scan(&'a mut self) -> Result<Vec<Token>> {
//        let mut tokens : Vec<Token> = Vec::new();
//        let mut chars = self.source.chars().peekable();
//
//        while let Some(t) = self.scan_token(&mut chars)? {
//            tokens.push(t);
//        }
//
//        tokens.push(self.static_token(Type::EOF, ""));
//        Ok(tokens)
//    }
//
//    fn scan_token(&'a self, src : &mut Peekable<Chars>) -> Result<Option<Token>> {
//        let mut lexeme : String = String::new();
//
//        while let Some(c) = src.next() {
//            use token::Type::*;
//            lexeme.push(c);
//            match c {
//                '(' => return Ok(Some(self.static_token(LeftParen, &lexeme))),
//                ')' => return Ok(Some(self.static_token(RightParen, &lexeme))),
//                '{' => return Ok(Some(self.static_token(LeftBrace, &lexeme))),
//                '}' => return Ok(Some(self.static_token(LeftBrace, &lexeme))),
//                ',' => return Ok(Some(self.static_token(Comma, &lexeme))),
//                '.' => return Ok(Some(self.static_token(Dot, &lexeme))),
//                '-' => return Ok(Some(self.static_token(Minus, &lexeme))),
//                '+' => return Ok(Some(self.static_token(Plus, &lexeme))),
//                ';' => return Ok(Some(self.static_token(Semicolon, &lexeme))),
//                '*' => return Ok(Some(self.static_token(Star, &lexeme))),
//                _ => return Err(Box::new(Error::Lexical(self.line, "unexpected char", ""))),
//            }
//        }
//
//        Ok(None)
//    }
//
//    fn static_token<'b>(&'a self, typ: Type, lex : &'b str) -> Token<'b> {
//        self.literal_token(typ, lex, None)
//    }
//
//    fn literal_token<'b>(&'a self, typ: Type, lex : &'b str, lit: Option<()>) -> Token<'b> {
//        Token {
//            typ: typ,
//            literal: lit,
//            lexeme: lex,
//            line: self.line,
//        }
//    }
//}
