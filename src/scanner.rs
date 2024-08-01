use crate::{error, token::Token, token_type::TokenType};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u64,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        return Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
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
            '\n' => self.line += 1,
            char => error(self.line, format!("Unexpected character: {char}")),
        }
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

    fn advance(&mut self) -> char {
        let char = self.source[self.current];
        self.current += 1;
        return char;
    }

    fn add_token(&mut self, r#type: TokenType, literal: Option<String>) {
        let text: String = self.source[self.start..self.current].into_iter().collect();
        self.tokens
            .push(Token::new(r#type, text, literal, self.line));
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }
}
