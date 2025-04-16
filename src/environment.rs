use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{RuntimeError, RuntimeExceptions},
    token::{LiteralValue, Token},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Environment {
    pub enclosing: Option<Rc<Environment>>,
    pub values: RefCell<HashMap<String, Option<LiteralValue>>>,
}

impl Environment {
    pub fn new(enclosing: Option<&Rc<Environment>>) -> Environment {
        Environment {
            enclosing: enclosing.map(|e| Rc::clone(e)),
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Option<LiteralValue>, RuntimeExceptions> {
        let value_ref = self.values.borrow();
        if value_ref.contains_key(&name.lexeme) {
            // cloning here isn't great, but using Rc<Environment> for closures (and objects?) ensures data update persistence
            return Ok(value_ref.get(&name.lexeme).unwrap().clone());
        }
        if self.enclosing.is_some() {
            return self.enclosing.as_ref().unwrap().get(name);
        }

        return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
            name,
            format!("Undefined variable '{}'.", name.lexeme).as_str(),
        )));
    }

    pub fn get_at(
        &self,
        distance: u64,
        name: &Token,
    ) -> Result<Option<LiteralValue>, RuntimeExceptions> {
        if distance > 0 {
            return self.enclosing.as_ref().unwrap().get_at(distance - 1, name);
        }
        return Ok(self.values.borrow().get(&name.lexeme).unwrap().clone());
    }

    pub fn assign(
        &self,
        name: &Token,
        value: Option<LiteralValue>,
    ) -> Result<(), RuntimeExceptions> {
        let mut value_ref = self.values.borrow_mut();
        if value_ref.contains_key(&name.lexeme) {
            value_ref.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if self.enclosing.is_some() {
            return self.enclosing.as_ref().unwrap().assign(name, value);
        }

        return Err(RuntimeExceptions::RuntimeError(RuntimeError::new(
            name,
            format!("Undefined variable '{}'.", name.lexeme).as_str(),
        )));
    }

    pub fn assign_at(
        &self,
        distance: u64,
        name: &Token,
        value: Option<LiteralValue>,
    ) -> Result<(), RuntimeExceptions> {
        if distance > 0 {
            return self
                .enclosing
                .as_ref()
                .unwrap()
                .assign_at(distance - 1, name, value);
        }
        self.values.borrow_mut().insert(name.lexeme.clone(), value);
        return Ok(());
    }

    pub fn define(&self, name: String, value: Option<LiteralValue>) {
        self.values.borrow_mut().insert(name, value);
    }
}
