use crate::{
    ast::{Binary, Expr, Grouping, Literal, Unary},
    error_token,
    token::{LiteralValue, Token},
    token_type::TokenType,
};

pub struct ParseError {}

impl ParseError {
    pub fn new() -> ParseError {
        return ParseError {};
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        return Parser { tokens, current: 0 };
    }

    pub fn parse(&mut self) -> Option<Expr> {
        return self.expression().ok();
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        return self.equality();
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.comparison()?;

        while self.r#match(&vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.comparison()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.term()?;

        while self.r#match(&vec![
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.term()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.factor()?;

        while self.r#match(&vec![TokenType::MINUS, TokenType::PLUS]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.factor()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.unary()?;

        while self.r#match(&vec![TokenType::SLASH, TokenType::STAR]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.unary()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.r#match(&vec![TokenType::BANG, TokenType::MINUS]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.unary()?;
            return Ok(Expr::Unary(Unary::new(operator, right)));
        }
        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.r#match(&vec![TokenType::FALSE]) {
            return Ok(Expr::Literal(Literal::new(Some(LiteralValue::Boolean(
                false,
            )))));
        }
        if self.r#match(&vec![TokenType::TRUE]) {
            return Ok(Expr::Literal(Literal::new(Some(LiteralValue::Boolean(
                true,
            )))));
        }
        if self.r#match(&vec![TokenType::NIL]) {
            return Ok(Expr::Literal(Literal::new(None)));
        }
        if self.r#match(&vec![TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expr::Literal(Literal::new(self.previous().literal.clone())));
        }
        if self.r#match(&vec![TokenType::LEFT_PAREN]) {
            let expr: Expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expressions.")?;
            return Ok(Expr::Grouping(Grouping::new(expr)));
        }
        return Err(self.error(self.peek(), "Expect expression."));
    }

    fn r#match(&mut self, types: &Vec<TokenType>) -> bool {
        for r#type in types {
            if self.check(r#type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn consume(&mut self, r#type: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(&r#type) {
            return Ok(self.advance());
        }
        return Err(self.error(self.peek(), message));
    }

    fn check(&self, r#type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return &self.peek().r#type == r#type;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn is_at_end(&self) -> bool {
        return self.peek().r#type == TokenType::EOF;
    }

    fn peek(&self) -> &Token {
        return self.tokens.get(self.current).unwrap();
    }

    fn previous(&self) -> &Token {
        return self.tokens.get(self.current - 1).unwrap();
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        error_token(token, message.to_string());
        return ParseError::new();
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().r#type == TokenType::SEMICOLON {
                return;
            }

            match self.peek().r#type {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
