use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::Environment,
    expr::{self, Expr},
    lox_callables::{LoxAnonymous, LoxCallable, LoxCallables, LoxFunction},
    runtime_error,
    stmt::{self, Stmt},
    token::{LiteralValue, Token},
    token_type::TokenType,
};

pub enum RuntimeExceptions {
    RuntimeError(RuntimeError),
    Return(Return),
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub token: Token,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> RuntimeError {
        RuntimeError {
            token: token.clone(),
            message: message.to_string(),
        }
    }
}

pub struct Return {
    pub value: Option<LiteralValue>,
}

impl Return {
    pub fn new(value: Option<LiteralValue>) -> Return {
        Return { value }
    }
}

pub struct Interpreter {
    pub globals: Rc<Environment>,
    environment: Rc<Environment>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(Environment::new(None));

        // native functions here
        globals.define(
            "clock".to_owned(),
            Some(LiteralValue::LoxCallable(LoxCallables::LoxAnonymous(
                Box::new(LoxAnonymous::new(
                    |_interpreter, _arguments| {
                        Ok(Some(LiteralValue::Number(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                        )))
                    },
                    || 0,
                )),
            ))),
        );

        let environment = Rc::clone(&globals);
        Interpreter {
            globals,
            environment,
        }
    }

    pub fn interpret_expr(&mut self, expression: Expr) {
        let value = self.evaluate(&Box::new(expression));
        if value.is_ok() {
            println!("{}", self.stringify(&value.ok().unwrap()));
            return;
        }
        match value.unwrap_err() {
            RuntimeExceptions::RuntimeError(run_error) => runtime_error(run_error),
            _ => {}
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        let mut error: Option<RuntimeExceptions> = None;
        for statement in statements {
            let result = self.execute(&statement);
            if result.is_err() {
                error = result.err();
                break;
            }
        }

        if error.is_some() {
            match error.unwrap() {
                RuntimeExceptions::RuntimeError(run_error) => runtime_error(run_error),
                _ => {}
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeExceptions> {
        stmt.accept(self)?;
        return Ok(());
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<Environment>,
    ) -> Result<(), RuntimeExceptions> {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;

        let mut error: Result<(), RuntimeExceptions> = Ok(());
        for statement in statements {
            let result = self.execute(statement);
            if result.is_err() {
                error = result;
                break;
            }
        }

        self.environment = previous;

        return error;
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Option<LiteralValue>, RuntimeExceptions> {
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
    ) -> Result<f64, RuntimeExceptions> {
        match operand {
            Some(LiteralValue::Number(value)) => return Ok(*value),
            _ => {
                return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
                    operator,
                    "Operand must be a number.",
                )))
            }
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &Option<LiteralValue>,
        right: &Option<LiteralValue>,
    ) -> Result<(f64, f64), RuntimeExceptions> {
        let lnumber = number_cast(left);
        let rnumber = number_cast(right);
        if lnumber.is_some() && rnumber.is_some() {
            return Ok((lnumber.unwrap(), rnumber.unwrap()));
        }
        return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
            operator,
            "Operands must be numbers.",
        )));
    }
}

impl stmt::Visitor for Interpreter {
    type Output = Result<(), RuntimeExceptions>;

    fn visit_block(&mut self, block: &stmt::Block) -> Self::Output {
        let result = self.execute_block(
            &block.statements,
            Rc::new(Environment::new(Some(&self.environment))),
        );
        return result;
    }

    fn visit_expression(&mut self, expression: &stmt::Expression) -> Self::Output {
        self.evaluate(&expression.expression)?;
        return Ok(());
    }

    fn visit_function(&mut self, function: &stmt::Function) -> Self::Output {
        let value = Some(LiteralValue::LoxCallable(LoxCallables::LoxFunction(
            Box::new(LoxFunction::new(
                function.clone(),
                Rc::clone(&self.environment),
            )),
        )));
        self.environment.define(function.name.lexeme.clone(), value);
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

    fn visit_return(&mut self, r#return: &stmt::Return) -> Self::Output {
        let mut value = None;
        if r#return.value.is_some() {
            value = self.evaluate(r#return.value.as_ref().unwrap())?;
        }

        return Err(RuntimeExceptions::Return(Return::new(value)));
    }

    fn visit_var(&mut self, var: &stmt::Var) -> Self::Output {
        let mut value: Option<LiteralValue> = None;
        if var.initializer.is_some() {
            value = self.evaluate(var.initializer.as_ref().unwrap())?;
        }

        self.environment.define(var.name.lexeme.clone(), value);
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
    type Output = Result<Option<LiteralValue>, RuntimeExceptions>;

    fn visit_assign(&mut self, assign: &expr::Assign) -> Self::Output {
        let value = self.evaluate(&assign.value)?;
        self.environment.assign(&assign.name, value.clone())?;
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

                return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
                    &binary.operator,
                    "Operands must be two numbers or two strings.",
                )));
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
                return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
                    &binary.operator,
                    "Invalid operator when evaluating binary!",
                )))
            }
        };
    }

    fn visit_call(&mut self, call: &expr::Call) -> Self::Output {
        let callee = self.evaluate(&call.callee)?;

        let mut arguments = Vec::new();
        for argument in &call.arguments {
            arguments.push(self.evaluate(&Box::new(argument))?);
        }

        let mut function = match callee {
            Some(LiteralValue::LoxCallable(callable)) => Ok(callable),
            _ => Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
                &call.paren,
                "Can only call functions and classes.",
            ))),
        }?;

        if arguments.len() != function.arity() {
            return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
                &call.paren,
                &format!(
                    "Expected {} arguments but got {}.",
                    function.arity(),
                    arguments.len()
                ),
            )));
        }

        let result = function.call(self, arguments);
        return match result {
            Err(RuntimeExceptions::Return(r#return)) => Ok(r#return.value),
            _ => result,
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
                return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
                    &logical.operator,
                    "Invalid operator when evaluating logical!",
                )))
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
                return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
                    &unary.operator,
                    "Invalid operator when evaluating unary!",
                )))
            }
        }
    }

    fn visit_variable(&mut self, variable: &expr::Variable) -> Self::Output {
        return Ok(self.environment.get(&variable.name)?);
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