use std::collections::HashMap;

use crate::{
    error,
    token::{Literal, Token},
    token_type::TokenType,
};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u64,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        return Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::from([
                ("and".to_string(), TokenType::AND),
                ("class".to_string(), TokenType::CLASS),
                ("else".to_string(), TokenType::ELSE),
                ("false".to_string(), TokenType::FALSE),
                ("for".to_string(), TokenType::FOR),
                ("fun".to_string(), TokenType::FUN),
                ("if".to_string(), TokenType::IF),
                ("nil".to_string(), TokenType::NIL),
                ("or".to_string(), TokenType::OR),
                ("print".to_string(), TokenType::PRINT),
                ("return".to_string(), TokenType::RETURN),
                ("super".to_string(), TokenType::SUPER),
                ("this".to_string(), TokenType::THIS),
                ("true".to_string(), TokenType::TRUE),
                ("var".to_string(), TokenType::VAR),
                ("while".to_string(), TokenType::WHILE),
            ]),
        };
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, "".to_string(), None, self.line));
        return &self.tokens;
    }

    fn scan_token(&mut self) {
        let char = self.advance();
        match char {
            '(' => self.add_token(TokenType::LEFT_PAREN, None),
            ')' => self.add_token(TokenType::RIGHT_PAREN, None),
            '{' => self.add_token(TokenType::LEFT_BRACE, None),
            '}' => self.add_token(TokenType::RIGHT_BRACE, None),
            ',' => self.add_token(TokenType::COMMA, None),
            '.' => self.add_token(TokenType::DOT, None),
            '-' => self.add_token(TokenType::MINUS, None),
            '+' => self.add_token(TokenType::PLUS, None),
            ';' => self.add_token(TokenType::SEMICOLON, None),
            '*' => self.add_token(TokenType::STAR, None),
            '!' => {
                let r#type = if self.r#match('=') {
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG
                };
                self.add_token(r#type, None);
            }
            '=' => {
                let r#type = if self.r#match('=') {
                    TokenType::EQUAL_EQUAL
                } else {
                    TokenType::EQUAL
                };
                self.add_token(r#type, None);
            }
            '<' => {
                let r#type = if self.r#match('=') {
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                };
                self.add_token(r#type, None);
            }
            '>' => {
                let r#type = if self.r#match('=') {
                    TokenType::GREATER_EQUAL
                } else {
                    TokenType::GREATER
                };
                self.add_token(r#type, None);
            }
            '/' => {
                if self.r#match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH, None);
                }
            }
            ' ' | '\r' | '\t' => {}
            '"' => self.string(),
            '\n' => self.line += 1,
            char => {
                if self.is_digit(char) {
                    self.number();
                } else if self.is_alpha(char) {
                    self.identifier();
                } else {
                    error(self.line, format!("Unexpected character: {char}"));
                }
            }
        }
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].into_iter().collect();
        let r#type = self.keywords.get(&text).unwrap_or(&TokenType::IDENTIFIER);
        self.add_token(r#type.clone(), None);
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let substring: String = self.source[self.start..self.current].into_iter().collect();
        self.add_token(
            TokenType::NUMBER,
            Some(Literal::Number(
                str::parse::<f64>(substring.as_str()).unwrap(),
            )),
        )
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            error(self.line, "Unterminated string.".to_string());
            return;
        }

        self.advance();

        let value: String = self.source[(self.start + 1)..(self.current - 1)]
            .into_iter()
            .collect();
        self.add_token(TokenType::STRING, Some(Literal::String(value)));
    }

    fn r#match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        return self.source[self.current];
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        return self.source[self.current + 1];
    }

    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        return self.is_alpha(c) || self.is_digit(c);
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn advance(&mut self) -> char {
        let char = self.source[self.current];
        self.current += 1;
        return char;
    }

    fn add_token(&mut self, r#type: TokenType, literal: Option<Literal>) {
        let text: String = self.source[self.start..self.current].into_iter().collect();
        self.tokens
            .push(Token::new(r#type, text, literal, self.line));
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }
}
