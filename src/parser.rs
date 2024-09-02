use crate::{
    error_token,
    expr::{Assign, Binary, Expr, Grouping, Literal, Unary, Variable},
    stmt::{Block, Expression, Print, Stmt, Var},
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

    pub fn parse(&mut self) -> Vec<Option<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            let statement = self.declaration();
            if statement.is_ok() {
                statements.push(statement.ok());
            } else {
                self.synchronize();
            }
        }

        return statements;
    }

    pub fn parse_expr(&mut self) -> Option<Expr> {
        return self.expression().ok();
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.r#match(&vec![TokenType::VAR]) {
            return Ok(self.var_declaration()?);
        }

        return Ok(self.statement()?);
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenType::IDENTIFIER, "Expect variable name.")?
            .clone();

        let mut initializer: Option<Expr> = None;
        if self.r#match(&vec![TokenType::EQUAL]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        )?;
        return Ok(Stmt::Var(Var::new(name, initializer)));
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.r#match(&vec![TokenType::PRINT]) {
            return Ok(self.print_statement()?);
        }
        if self.r#match(&vec![TokenType::LEFT_BRACE]) {
            return Ok(Stmt::Block(Block::new(self.block()?)));
        }

        return Ok(self.expression_statement()?);
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        return Ok(Stmt::Print(Print::new(value)));
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.")?;
        return Ok(statements);
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.")?;
        return Ok(Stmt::Expression(Expression::new(expr)));
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        return self.assignment();
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;
        if self.r#match(&vec![TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(variable) => {
                    return Ok(Expr::Assign(Assign::new(variable.name, value)));
                }
                _ => {}
            }

            self.error(&equals, "Invalid assignment target.");
        }

        return Ok(expr);
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
        if self.r#match(&vec![TokenType::IDENTIFIER]) {
            return Ok(Expr::Variable(Variable::new(self.previous().clone())));
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
