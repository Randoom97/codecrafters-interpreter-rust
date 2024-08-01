use std::fmt::Display;

use crate::token_type::TokenType;

pub enum Literal {
    String(String),
    Number(f64),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(value) => write!(f, "{}", value),
            Literal::Number(value) => write!(f, "{:?}", value),
        }
    }
}

pub struct Token {
    r#type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: u64,
}

impl Token {
    pub fn new(r#type: TokenType, lexeme: String, literal: Option<Literal>, line: u64) -> Token {
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
                .unwrap_or(&Literal::String("null".to_string()))
        );
    }
}
