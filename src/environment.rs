use std::collections::HashMap;

use crate::{
    interpreter::RuntimeError,
    token::{LiteralValue, Token},
};

pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, Option<LiteralValue>>,
}

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Environment {
        Environment {
            enclosing: enclosing.map(|e| Box::new(e)),
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<&Option<LiteralValue>, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            return Ok(self.values.get(&name.lexeme).unwrap());
        }
        if self.enclosing.is_some() {
            return self.enclosing.as_ref().unwrap().get(name);
        }

        return Err(RuntimeError::new(
            name,
            format!("Undefined variable '{}'.", name.lexeme).as_str(),
        ));
    }

    pub fn assign(
        &mut self,
        name: &Token,
        value: Option<LiteralValue>,
    ) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if self.enclosing.is_some() {
            return self.enclosing.as_mut().unwrap().assign(name, value);
        }

        return Err(RuntimeError::new(
            name,
            format!("Undefined variable '{}'.", name.lexeme).as_str(),
        ));
    }

    pub fn define(&mut self, name: String, value: Option<LiteralValue>) {
        self.values.insert(name, value);
    }
}
