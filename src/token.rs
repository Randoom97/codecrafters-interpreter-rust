use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use crate::{lox_callables::LoxCallables, lox_instance::LoxInstance, token_type::TokenType};

#[derive(Clone, PartialEq, Debug)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    LoxCallable(LoxCallables),
    LoxInstance(LoxInstance),
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::String(value) => write!(f, "{}", value),
            LiteralValue::Number(value) => write!(f, "{:?}", value),
            LiteralValue::Boolean(value) => write!(f, "{}", value),
            LiteralValue::LoxCallable(value) => write!(f, "{}", value),
            LiteralValue::LoxInstance(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: u64,
    pub col: u64,
}

impl Token {
    pub fn new(
        r#type: TokenType,
        lexeme: String,
        literal: Option<LiteralValue>,
        line: u64,
        col: u64,
    ) -> Token {
        return Token {
            r#type,
            lexeme,
            literal,
            line,
            col,
        };
    }

    pub fn to_string(&self) -> String {
        return format!(
            "{} {} {}",
            self.r#type,
            self.lexeme,
            self.literal
                .as_ref()
                .unwrap_or(&LiteralValue::String("null".to_string()))
        );
    }
}

impl Eq for Token {}

impl Hash for Token {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        self.lexeme.hash(state);
        // ingore literal value because of type issues, and it's technically included in the lexeme
        self.line.hash(state);
        self.col.hash(state);
    }
}
