use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::{Interpreter, RuntimeExceptions},
    lox_instance::LoxInstance,
    stmt::{self},
    token::LiteralValue,
};

pub trait LoxCallable {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Option<LiteralValue>>,
    ) -> Result<Option<LiteralValue>, RuntimeExceptions>;
    fn arity(&self) -> usize;
}

#[derive(Clone, PartialEq, Debug)]
pub enum LoxCallables {
    LoxFunction(Box<LoxFunction>),
    LoxAnonymous(Box<LoxAnonymous>),
    LoxClass(LoxClass),
}

impl Display for LoxCallables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxCallables::LoxAnonymous(_) => write!(f, "<anonymous function>"),
            LoxCallables::LoxFunction(function) => {
                write!(f, "<fn {}>", function.declaration.name.lexeme)
            }
            LoxCallables::LoxClass(class) => write!(f, "{}", class.name),
        }
    }
}

impl LoxCallable for LoxCallables {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Option<LiteralValue>>,
    ) -> Result<Option<LiteralValue>, RuntimeExceptions> {
        match self {
            LoxCallables::LoxFunction(value) => value.call(interpreter, arguments),
            LoxCallables::LoxAnonymous(value) => value.call(interpreter, arguments),
            LoxCallables::LoxClass(value) => value.call(interpreter, arguments),
        }
    }

    fn arity(&self) -> usize {
        match self {
            LoxCallables::LoxFunction(value) => value.arity(),
            LoxCallables::LoxAnonymous(value) => value.arity(),
            LoxCallables::LoxClass(value) => value.arity(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LoxAnonymous {
    // maybe call this native function if it doesn't get reused
    call_ref: fn(
        &mut Interpreter,
        Vec<Option<LiteralValue>>,
    ) -> Result<Option<LiteralValue>, RuntimeExceptions>,
    arity_ref: fn() -> usize,
}

impl LoxAnonymous {
    pub fn new(
        call: fn(
            &mut Interpreter,
            Vec<Option<LiteralValue>>,
        ) -> Result<Option<LiteralValue>, RuntimeExceptions>,
        arity: fn() -> usize,
    ) -> LoxAnonymous {
        LoxAnonymous {
            call_ref: call,
            arity_ref: arity,
        }
    }
}

impl LoxCallable for LoxAnonymous {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Option<LiteralValue>>,
    ) -> Result<Option<LiteralValue>, RuntimeExceptions> {
        (self.call_ref)(interpreter, arguments)
    }

    fn arity(&self) -> usize {
        (self.arity_ref)()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LoxFunction {
    declaration: stmt::Function,
    closure: Rc<Environment>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: stmt::Function,
        closure: Rc<Environment>,
        is_initializer: bool,
    ) -> LoxFunction {
        LoxFunction {
            declaration,
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: LoxInstance) -> LoxFunction {
        let environment = Rc::new(Environment::new(Some(&self.closure)));
        environment.define(
            "this".to_string(),
            Some(LiteralValue::LoxInstance(instance.clone())),
        );
        return LoxFunction::new(self.declaration.clone(), environment, self.is_initializer);
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Option<LiteralValue>>,
    ) -> Result<Option<LiteralValue>, RuntimeExceptions> {
        let environment = Rc::new(Environment::new(Some(&self.closure)));
        for i in 0..self.declaration.params.len() {
            environment.define(
                self.declaration.params.get(i).unwrap().lexeme.clone(),
                arguments.get(i).unwrap().clone(),
            );
        }

        let result = interpreter
            .execute_block(&self.declaration.body, environment)
            .map(|_| None); // convert Ok from type '()' to 'Option<Literal>'
        match result.as_ref().err() {
            Some(RuntimeExceptions::Return(r#return)) => {
                if self.is_initializer {
                    return self.closure.get_at(0, &"this".to_string());
                }

                return Ok(r#return.value.clone());
            }
            _ => {}
        }

        if self.is_initializer {
            return self.closure.get_at(0, &"this".to_string());
        }

        return result;
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Box<LoxClass>>,
    pub methods: Rc<HashMap<String, LoxFunction>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> LoxClass {
        LoxClass {
            name,
            superclass: superclass.map(|sc| Box::new(sc)),
            methods: Rc::new(methods),
        }
    }

    pub fn find_method(&self, name: &String) -> Option<LoxFunction> {
        let method = self.methods.get(name);
        if method.is_some() {
            return method.cloned();
        }

        if self.superclass.is_some() {
            return self.superclass.as_ref().unwrap().find_method(name);
        }

        return None;
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Option<LiteralValue>>,
    ) -> Result<Option<LiteralValue>, RuntimeExceptions> {
        let instance = LoxInstance::new(self.clone());
        let initializer = self.find_method(&"init".to_string());
        if initializer.is_some() {
            initializer
                .unwrap()
                .bind(instance.clone())
                .call(interpreter, arguments)?;
        }
        return Ok(Some(LiteralValue::LoxInstance(instance)));
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method(&"init".to_string());
        if initializer.is_none() {
            return 0;
        }
        return initializer.unwrap().arity();
    }
}
