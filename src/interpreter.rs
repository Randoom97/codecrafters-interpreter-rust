use crate::{
    ast::{Expr, Visitor},
    runtime_error,
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

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Interpreter {
        return Interpreter {};
    }

    pub fn interpret(&self, expression: Expr) {
        let value = self.evaluate(&Box::new(expression));
        if value.is_ok() {
            println!("{}", self.stringify(&value.as_ref().unwrap()));
            return;
        }
        runtime_error(value.unwrap_err());
    }

    fn evaluate(&self, expr: &Box<Expr>) -> Result<Option<LiteralValue>, RuntimeError> {
        return expr.accept(self);
    }

    fn is_truthy(&self, value: Option<LiteralValue>) -> bool {
        if value.is_none() {
            return false;
        }
        match value.unwrap() {
            LiteralValue::Boolean(value) => return value,
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

impl Visitor for Interpreter {
    type Output = Result<Option<LiteralValue>, RuntimeError>;

    fn visit_binary(&self, binary: &crate::ast::Binary) -> Self::Output {
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
            // Unreachable
            _ => return Ok(None),
        };
    }

    fn visit_grouping(&self, grouping: &crate::ast::Grouping) -> Self::Output {
        return self.evaluate(&grouping.expression);
    }

    fn visit_literal(&self, literal: &crate::ast::Literal) -> Self::Output {
        return Ok(literal.value.clone());
    }

    fn visit_unary(&self, unary: &crate::ast::Unary) -> Self::Output {
        let right = self.evaluate(&unary.right)?;

        match unary.operator.r#type {
            TokenType::MINUS => {
                let number = self.check_number_operand(&unary.operator, &right)?;
                return Ok(Some(LiteralValue::Number(-number)));
            }
            TokenType::BANG => return Ok(Some(LiteralValue::Boolean(!self.is_truthy(right)))),
            // Unreachable
            _ => return Ok(None),
        }
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
