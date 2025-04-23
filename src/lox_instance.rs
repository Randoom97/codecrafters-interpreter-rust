use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    interpreter::{RuntimeError, RuntimeExceptions},
    lox_callables::{LoxCallables, LoxClass},
    token::{LiteralValue, Token},
};

#[derive(Clone, PartialEq, Debug)]
pub struct LoxInstance {
    klass: LoxClass,
    fields: Rc<RefCell<HashMap<String, Option<LiteralValue>>>>,
}

impl LoxInstance {
    pub fn new(klass: LoxClass) -> LoxInstance {
        LoxInstance {
            klass,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Option<LiteralValue>, RuntimeExceptions> {
        let fields_ref = self.fields.borrow();
        if fields_ref.contains_key(&name.lexeme) {
            return Ok(fields_ref.get(&name.lexeme).unwrap().clone());
        }

        let method = self.klass.find_method(&name.lexeme);
        if method.is_some() {
            return Ok(Some(LiteralValue::LoxCallable(LoxCallables::LoxFunction(
                Box::new(method.unwrap().bind(self.clone())),
            ))));
        }

        return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
            name,
            &format!("Undefined property '{}'.", name.lexeme),
        )));
    }

    pub fn set(&self, name: &Token, value: Option<LiteralValue>) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}
