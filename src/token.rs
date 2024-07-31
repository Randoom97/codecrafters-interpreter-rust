use crate::token_type::TokenType;

pub struct Token {
    r#type: TokenType,
    lexeme: String,
    literal: Option<String>,
    line: u64,
}

impl Token {
    pub fn new(r#type: TokenType, lexeme: String, literal: Option<String>, line: u64) -> Token {
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
            self.literal.as_ref().unwrap_or(&"null".to_string())
        );
    }
}
