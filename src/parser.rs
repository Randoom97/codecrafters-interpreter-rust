use crate::{
    error_token,
    expr::{
        Assign, Binary, Call, Expr, Get, Grouping, Literal, Logical, Set, This, Unary, Variable,
    },
    stmt::{Block, Class, Expression, Function, If, Print, Return, Stmt, Var, While},
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
        if self.r#match(&vec![TokenType::CLASS]) {
            return self.class_declaration().map(|r| Stmt::Class(r));
        }
        if self.r#match(&vec![TokenType::FUN]) {
            return self.function("function").map(|r| Stmt::Function(r));
        }
        if self.r#match(&vec![TokenType::VAR]) {
            return self.var_declaration().map(|r| Stmt::Var(r));
        }

        return self.statement();
    }

    fn class_declaration(&mut self) -> Result<Class, ParseError> {
        let name = self
            .consume(TokenType::IDENTIFIER, "Expect class name.")?
            .clone();
        self.consume(TokenType::LEFT_BRACE, "Expect '{' before class body.")?;

        let mut methods = Vec::new();
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after class body.")?;

        return Ok(Class::new(name, methods));
    }

    fn var_declaration(&mut self) -> Result<Var, ParseError> {
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
        return Ok(Var::new(name, initializer));
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.r#match(&vec![TokenType::FOR]) {
            return self.for_statement();
        }
        if self.r#match(&vec![TokenType::IF]) {
            return self.if_statement().map(|r| Stmt::If(r));
        }
        if self.r#match(&vec![TokenType::PRINT]) {
            return self.print_statement().map(|r| Stmt::Print(r));
        }
        if self.r#match(&vec![TokenType::RETURN]) {
            return self.return_statement().map(|r| Stmt::Return(r));
        }
        if self.r#match(&vec![TokenType::WHILE]) {
            return self.while_statement().map(|r| Stmt::While(r));
        }
        if self.r#match(&vec![TokenType::LEFT_BRACE]) {
            return self.block().map(|r| Stmt::Block(Block::new(r)));
        }

        return self.expression_statement().map(|r| Stmt::Expression(r));
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'.")?;

        let mut initializer: Option<Stmt> = None;
        if self.r#match(&vec![TokenType::SEMICOLON]) {
            // no initializer
        } else if self.r#match(&vec![TokenType::VAR]) {
            initializer = Some(Stmt::Var(self.var_declaration()?));
        } else {
            initializer = Some(Stmt::Expression(self.expression_statement()?));
        }

        let mut condition: Option<Expr> = None;
        if !self.check(&TokenType::SEMICOLON) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.")?;

        let mut increment: Option<Expr> = None;
        if !self.check(&TokenType::RIGHT_PAREN) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if increment.is_some() {
            body = Stmt::Block(Block::new(vec![
                body,
                Stmt::Expression(Expression::new(increment.unwrap())),
            ]));
        }
        let mut r#while = Stmt::While(While::new(
            condition.unwrap_or(Expr::Literal(Literal::new(Some(LiteralValue::Boolean(
                true,
            ))))),
            body,
        ));
        if initializer.is_some() {
            r#while = Stmt::Block(Block::new(vec![initializer.unwrap(), r#while]));
        }

        return Ok(r#while);
    }

    fn if_statement(&mut self) -> Result<If, ParseError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after if condition.")?;
        let then_branch = self.statement()?;
        let mut else_branch: Option<Stmt> = None;
        if self.r#match(&vec![TokenType::ELSE]) {
            else_branch = Some(self.statement()?);
        }

        return Ok(If::new(condition, then_branch, else_branch));
    }

    fn print_statement(&mut self) -> Result<Print, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        return Ok(Print::new(value));
    }

    fn return_statement(&mut self) -> Result<Return, ParseError> {
        let keyword = self.previous().clone();
        let mut value = None;
        if !self.check(&TokenType::SEMICOLON) {
            value = Some(self.expression()?);
        }

        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.")?;
        return Ok(Return::new(keyword, value));
    }

    fn while_statement(&mut self) -> Result<While, ParseError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition.")?;
        let body = self.statement()?;

        return Ok(While::new(condition, body));
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.")?;
        return Ok(statements);
    }

    fn expression_statement(&mut self) -> Result<Expression, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.")?;
        return Ok(Expression::new(expr));
    }

    fn function(&mut self, kind: &str) -> Result<Function, ParseError> {
        let name = self
            .consume(TokenType::IDENTIFIER, &format!("Expect {kind} name."))?
            .clone();
        self.consume(
            TokenType::LEFT_PAREN,
            &format!("Expect '(' after {kind} name."),
        )?;
        let mut parameters = Vec::new();
        if !self.check(&TokenType::RIGHT_PAREN) {
            loop {
                if parameters.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 parameters.");
                }

                parameters.push(
                    self.consume(TokenType::IDENTIFIER, "Expect parameter name.")?
                        .clone(),
                );

                if !self.r#match(&vec![TokenType::COMMA]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after parameters.")?;
        self.consume(
            TokenType::LEFT_BRACE,
            &format!("Expect '{{' before {kind} body."),
        )?;
        let body = self.block()?;

        return Ok(Function::new(name, parameters, body));
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        return self.assignment();
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;
        if self.r#match(&vec![TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(variable) => {
                    return Ok(Expr::Assign(Assign::new(variable.name, value)));
                }
                Expr::Get(get) => {
                    return Ok(Expr::Set(Set::new(*get.object, get.name, value)));
                }
                _ => {}
            }

            self.error(&equals, "Invalid assignment target.");
        }

        return Ok(expr);
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.r#match(&vec![TokenType::OR]) {
            let operator: Token = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical(Logical::new(expr, operator, right));
        }

        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.r#match(&vec![TokenType::AND]) {
            let operator: Token = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical(Logical::new(expr, operator, right));
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
        return self.call();
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.r#match(&vec![TokenType::LEFT_PAREN]) {
                expr = self.finish_call(expr)?;
            } else if self.r#match(&vec![TokenType::DOT]) {
                let name =
                    self.consume(TokenType::IDENTIFIER, "Expect property name after '.'.")?;
                expr = Expr::Get(Get::new(expr, name.clone()));
            } else {
                break;
            }
        }

        return Ok(expr);
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);

                if !self.r#match(&vec![TokenType::COMMA]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.")?;

        return Ok(Expr::Call(Call::new(callee, paren.to_owned(), arguments)));
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
        if self.r#match(&vec![TokenType::THIS]) {
            return Ok(Expr::This(This::new(self.previous().clone())));
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
        error_token(token, message);
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
