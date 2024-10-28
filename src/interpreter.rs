use crate::{
    environment::Environment,
    expr::{self, Expr},
    runtime_error,
    stmt::{self, Stmt},
    token::{LiteralValue, Token},
    token_type::TokenType,
};

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub token: Token,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> RuntimeError {
        return RuntimeError {
            token: token.clone(),
            message: message.to_string(),
        };
    }
}

pub struct Interpreter {
    environment: Option<Environment>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Some(Environment::new(None)),
        }
    }

    pub fn interpret_expr(&mut self, expression: Expr) {
        let value = self.evaluate(&Box::new(expression));
        if value.is_ok() {
            println!("{}", self.stringify(&value.as_ref().unwrap()));
            return;
        }
        runtime_error(value.unwrap_err());
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        let mut error: Option<RuntimeError> = None;
        for statement in statements {
            let result = self.execute(&statement);
            if result.is_err() {
                error = result.err();
                break;
            }
        }

        if error.is_some() {
            runtime_error(error.unwrap());
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)?;
        return Ok(());
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>) -> Result<(), RuntimeError> {
        self.environment = Some(Environment::new(self.environment.take()));

        let mut error: Result<(), RuntimeError> = Ok(());
        for statement in statements {
            let result = self.execute(statement);
            if result.is_err() {
                error = result;
                break;
            }
        }

        self.environment = self
            .environment
            .as_mut()
            .unwrap()
            .enclosing
            .take()
            .map(|e| *e);

        return error;
    }

    fn evaluate(&mut self, expr: &Box<Expr>) -> Result<Option<LiteralValue>, RuntimeError> {
        return expr.accept(self);
    }

    fn is_truthy(&self, value: &Option<LiteralValue>) -> bool {
        if value.is_none() {
            return false;
        }
        match value.as_ref().unwrap() {
            LiteralValue::Boolean(value) => return *value,
            _ => return true,
        }
    }

    fn is_equal(&self, a: &Option<LiteralValue>, b: &Option<LiteralValue>) -> bool {
        return a == b;
    }

    fn stringify(&self, value: &Option<LiteralValue>) -> String {
        if value.is_none() {
            return "nil".to_string();
        }

        return match value.as_ref().unwrap() {
            LiteralValue::Number(_) => value
                .as_ref()
                .unwrap()
                .to_string()
                .trim_end_matches(".0")
                .to_string(),
            _ => value.as_ref().unwrap().to_string(),
        };
    }

    fn check_number_operand(
        &self,
        operator: &Token,
        operand: &Option<LiteralValue>,
    ) -> Result<f64, RuntimeError> {
        match operand {
            Some(LiteralValue::Number(value)) => return Ok(*value),
            _ => return Err(RuntimeError::new(operator, "Operand must be a number.")),
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &Option<LiteralValue>,
        right: &Option<LiteralValue>,
    ) -> Result<(f64, f64), RuntimeError> {
        let lnumber = number_cast(left);
        let rnumber = number_cast(right);
        if lnumber.is_some() && rnumber.is_some() {
            return Ok((lnumber.unwrap(), rnumber.unwrap()));
        }
        return Err(RuntimeError::new(operator, "Operands must be numbers."));
    }
}

impl stmt::Visitor for Interpreter {
    type Output = Result<(), RuntimeError>;

    fn visit_block(&mut self, block: &stmt::Block) -> Self::Output {
        self.execute_block(&block.statements)?;
        return Ok(());
    }

    fn visit_expression(&mut self, expression: &stmt::Expression) -> Self::Output {
        self.evaluate(&expression.expression)?;
        return Ok(());
    }

    fn visit_if(&mut self, r#if: &stmt::If) -> Self::Output {
        let condition_value = self.evaluate(&r#if.condition)?;
        if self.is_truthy(&condition_value) {
            self.execute(&r#if.then_branch)?;
        } else if r#if.else_branch.is_some() {
            self.execute(r#if.else_branch.as_ref().unwrap())?;
        }
        return Ok(());
    }

    fn visit_print(&mut self, print: &stmt::Print) -> Self::Output {
        let value = self.evaluate(&print.expression)?;
        println!("{}", self.stringify(&value));
        return Ok(());
    }

    fn visit_var(&mut self, var: &stmt::Var) -> Self::Output {
        let mut value: Option<LiteralValue> = None;
        if var.initializer.is_some() {
            value = self.evaluate(var.initializer.as_ref().unwrap())?;
        }

        self.environment
            .as_mut()
            .unwrap()
            .define(var.name.lexeme.clone(), value);
        return Ok(());
    }

    fn visit_while(&mut self, r#while: &stmt::While) -> Self::Output {
        let mut condition_value = self.evaluate(&r#while.condition)?;
        while self.is_truthy(&condition_value) {
            self.execute(&r#while.body)?;
            condition_value = self.evaluate(&r#while.condition)?;
        }

        return Ok(());
    }
}

impl expr::Visitor for Interpreter {
    type Output = Result<Option<LiteralValue>, RuntimeError>;

    fn visit_assign(&mut self, assign: &expr::Assign) -> Self::Output {
        let value = self.evaluate(&assign.value)?;
        self.environment
            .as_mut()
            .unwrap()
            .assign(&assign.name, value.clone())?;
        return Ok(value);
    }

    fn visit_binary(&mut self, binary: &expr::Binary) -> Self::Output {
        let left = self.evaluate(&binary.left)?;
        let right = self.evaluate(&binary.right)?;

        match binary.operator.r#type {
            TokenType::MINUS => {
                let (lnumber, rnumber) =
                    self.check_number_operands(&binary.operator, &left, &right)?;
                return Ok(Some(LiteralValue::Number(lnumber - rnumber)));
            }
            TokenType::SLASH => {
                let (lnumber, rnumber) =
                    self.check_number_operands(&binary.operator, &left, &right)?;
                return Ok(Some(LiteralValue::Number(lnumber / rnumber)));
            }
            TokenType::STAR => {
                let (lnumber, rnumber) =
                    self.check_number_operands(&binary.operator, &left, &right)?;
                return Ok(Some(LiteralValue::Number(lnumber * rnumber)));
            }
            TokenType::PLUS => {
                let lnumber = number_cast(&left);
                let rnumber = number_cast(&right);
                if lnumber.is_some() && rnumber.is_some() {
                    return Ok(Some(LiteralValue::Number(
                        lnumber.unwrap() + rnumber.unwrap(),
                    )));
                }

                let lstring = string_cast(&left);
                let rstring = string_cast(&right);
                if lstring.is_some() && rstring.is_some() {
                    return Ok(Some(LiteralValue::String(
                        lstring.unwrap() + rstring.unwrap().as_str(),
                    )));
                }

                return Err(RuntimeError::new(
                    &binary.operator,
                    "Operands must be two numbers or two strings.",
                ));
            }
            TokenType::GREATER => {
                let (lnumber, rnumber) =
                    self.check_number_operands(&binary.operator, &left, &right)?;
                return Ok(Some(LiteralValue::Boolean(lnumber > rnumber)));
            }
            TokenType::GREATER_EQUAL => {
                let (lnumber, rnumber) =
                    self.check_number_operands(&binary.operator, &left, &right)?;
                return Ok(Some(LiteralValue::Boolean(lnumber >= rnumber)));
            }
            TokenType::LESS => {
                let (lnumber, rnumber) =
                    self.check_number_operands(&binary.operator, &left, &right)?;
                return Ok(Some(LiteralValue::Boolean(lnumber < rnumber)));
            }
            TokenType::LESS_EQUAL => {
                let (lnumber, rnumber) =
                    self.check_number_operands(&binary.operator, &left, &right)?;
                return Ok(Some(LiteralValue::Boolean(lnumber <= rnumber)));
            }
            TokenType::BANG_EQUAL => {
                return Ok(Some(LiteralValue::Boolean(!self.is_equal(&left, &right))))
            }
            TokenType::EQUAL_EQUAL => {
                return Ok(Some(LiteralValue::Boolean(self.is_equal(&left, &right))))
            }
            _ => {
                return Err(RuntimeError::new(
                    &binary.operator,
                    "Invalid operator when evaluating binary!",
                ))
            }
        };
    }

    fn visit_grouping(&mut self, grouping: &expr::Grouping) -> Self::Output {
        return self.evaluate(&grouping.expression);
    }

    fn visit_literal(&mut self, literal: &expr::Literal) -> Self::Output {
        return Ok(literal.value.clone());
    }

    fn visit_logical(&mut self, logical: &expr::Logical) -> Self::Output {
        let left = self.evaluate(&logical.left)?;

        let left_truthy = self.is_truthy(&left);
        match logical.operator.r#type {
            TokenType::OR => {
                if left_truthy {
                    return Ok(left);
                }
            }
            TokenType::AND => {
                if !left_truthy {
                    return Ok(left);
                }
            }
            _ => {
                return Err(RuntimeError::new(
                    &logical.operator,
                    "Invalid operator when evaluating logical!",
                ))
            }
        }

        return self.evaluate(&logical.right);
    }

    fn visit_unary(&mut self, unary: &expr::Unary) -> Self::Output {
        let right = self.evaluate(&unary.right)?;

        match unary.operator.r#type {
            TokenType::MINUS => {
                let number = self.check_number_operand(&unary.operator, &right)?;
                return Ok(Some(LiteralValue::Number(-number)));
            }
            TokenType::BANG => return Ok(Some(LiteralValue::Boolean(!self.is_truthy(&right)))),
            _ => {
                return Err(RuntimeError::new(
                    &unary.operator,
                    "Invalid operator when evaluating unary!",
                ))
            }
        }
    }

    fn visit_variable(&mut self, variable: &expr::Variable) -> Self::Output {
        return Ok(self
            .environment
            .as_ref()
            .unwrap()
            .get(&variable.name)?
            .clone());
    }
}

fn number_cast(value: &Option<LiteralValue>) -> Option<f64> {
    return match value {
        Some(LiteralValue::Number(value)) => Some(*value),
        _ => None,
    };
}

fn string_cast(value: &Option<LiteralValue>) -> Option<String> {
    return match value {
        Some(LiteralValue::String(value)) => Some(value.clone()),
        _ => None,
    };
}
