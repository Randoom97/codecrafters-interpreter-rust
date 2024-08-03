use std::fmt::Display;

use crate::token_type::TokenType;

#[derive(Clone)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::String(value) => write!(f, "{}", value),
            LiteralValue::Number(value) => write!(f, "{:?}", value),
            LiteralValue::Boolean(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: u64,
}

impl Token {
    pub fn new(
        r#type: TokenType,
        lexeme: String,
        literal: Option<LiteralValue>,
        line: u64,
    ) -> Token {
        return Token {
            r#type,
            lexeme,
            literal,
            line,
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
