use std::rc::Rc;
use crate::token::{InnerToken, Token, TokenType, Lexeme, ErrorType};

pub struct Scanner {
    inner: InnerToken,
}

impl Scanner {
    pub fn new(source: &Rc<String>, line: usize) -> Self {
        let lex = Lexeme::new(source, 0, 0);
        let inner = InnerToken::new(line, lex);
        Self { inner }
    }

    pub fn debug(self) {
        let mut line = 0;
        for tkn in self {
            if tkn.ln() != line {
                line = tkn.ln();
                eprint!("{:04}", tkn.ln())
            } else {
                eprint!("    ")
            };

            eprintln!(":{:03} | {:>12} | {:?}", tkn.col(), format!("{:?}", tkn.typ()), tkn.lex());
        }
    }

    fn at_end(&self) -> bool { self.inner.at_end() }

    fn error(&mut self, err: ErrorType) -> Option<Token> {
        self.token(TokenType::Error(err))
    }

    fn token(&mut self, typ: TokenType) -> Option<Token> {
        Some(Token::new(typ, self.inner.clone()))
    }

    fn shift(&mut self) { self.inner.lex.shift() }

    fn advance(&mut self) -> char { self.inner.lex.adv_char() }

    fn matches(&mut self, c: char) -> bool { self.inner.lex.matches(c) }

    fn matches_or(&mut self, c: char, ok: TokenType, or: TokenType) -> Option<Token> {
        let typ = if self.matches(c) { ok } else { or };
        self.token(typ)
    }

    fn skip_whitespace(&mut self) {
        while !self.at_end() {
            match self.inner.lex.peek() {
                ' ' | '\r' | '\t' => { self.inner.lex.adv(); }
                _ => break,
            }
        }
    }

    fn consume_line_comment(&mut self) {
        while !self.at_end() && self.inner.lex.peek() != '\n' {
            self.inner.lex.adv();
        }
    }

    fn consume_block_comment(&mut self) {
        while !self.at_end() {
            if '*' == self.advance() && self.matches('/') {
                return;
            }
        }
    }

    fn string(&mut self) -> Option<Token> {
        let mut escaped = false;
        while !self.at_end() {
            match self.advance() {
                '"' if !escaped => return self.token(TokenType::String),
                '\\' if !escaped => escaped = true,
                _ => escaped = false,
            };
        }

        self.error(ErrorType::UnterminatedString)
    }

    fn number(&mut self) -> Option<Token> {
        while char::is_ascii_digit(&self.inner.lex.peek()) {
            self.inner.lex.adv();
        }

        if self.inner.lex.peek() == '.' && char::is_ascii_digit(&self.inner.lex.peek_next()) {
            self.inner.lex.adv();
            self.inner.lex.adv();
            while char::is_ascii_digit(&self.inner.lex.peek()) {
                self.inner.lex.adv();
            }
        }

        self.token(TokenType::Number)
    }

    fn identifier(&mut self) -> Option<Token> {
        while char::is_ascii_alphanumeric(&self.inner.lex.peek()) {
            self.inner.lex.adv();
        }

        if let Some(typ) = TokenType::reserved(self.inner.lex.value()) {
            return self.token(typ);
        }

        self.token(TokenType::Identifier)
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.skip_whitespace();
        self.shift();

        if self.at_end() { return None; }

        match self.advance() {
            // Single Chars
            '(' => self.token(TokenType::LeftParen),
            ')' => self.token(TokenType::RightParen),
            '{' => self.token(TokenType::LeftBrace),
            '}' => self.token(TokenType::RightBrace),
            ';' => self.token(TokenType::Semicolon),
            ',' => self.token(TokenType::Comma),
            '.' => self.token(TokenType::Dot),
            '-' => self.token(TokenType::Minus),
            '+' => self.token(TokenType::Plus),
            '*' => self.token(TokenType::Star),

            '!' => self.matches_or('=', TokenType::BangEqual, TokenType::Bang),
            '=' => self.matches_or('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.matches_or('=', TokenType::LessEqual, TokenType::Less),
            '>' => self.matches_or('=', TokenType::GreaterEqual, TokenType::Greater),

            '"' => self.string(),
            d if d.is_ascii_digit() => self.number(),
            i if i.is_ascii_alphabetic() => self.identifier(),

            '/' => if self.matches('/') {
                self.consume_line_comment();
                self.next()
            } else if self.matches('*') {
                self.consume_block_comment();
                self.next()
            } else {
                self.token(TokenType::Slash)
            },

            '\n' => {
                self.inner.incr_line();
                self.next()
            }

            _ => self.error(ErrorType::UnexpectedChar),
        }
    }
}
