use std::collections::HashMap;

use crate::{
    error_token,
    expr::{self, Expr},
    interpreter::Interpreter,
    stmt::{self, Stmt},
    token::Token,
};

#[derive(Clone)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmts(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        stmt.accept(self);
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        expr.accept(self);
    }

    fn resolve_function(&mut self, function: &stmt::Function, r#type: FunctionType) {
        let enclosing_function = self.current_function.clone();
        self.current_function = r#type;

        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(&function.body);
        self.end_scope();

        self.current_function = enclosing_function;
    }

    fn resolve_local(&mut self, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(name, (self.scopes.len() - 1 - i) as u64);
                return;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        let scope_option = self.scopes.last_mut();
        if scope_option.is_none() {
            return;
        }

        let scope = scope_option.unwrap();

        if scope.contains_key(&name.lexeme) {
            error_token(name, "Already a variable with this name in this scope.");
        }

        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        let scope_option = self.scopes.last_mut();
        if scope_option.is_none() {
            return;
        }

        let scope = scope_option.unwrap();
        scope.insert(name.lexeme.clone(), true);
    }
}

impl stmt::Visitor for Resolver {
    type Output = ();

    fn visit_block(&mut self, block: &stmt::Block) -> Self::Output {
        self.begin_scope();
        self.resolve_stmts(&block.statements);
        self.end_scope();
    }

    fn visit_class(&mut self, class: &stmt::Class) -> Self::Output {
        let enclosing_class = self.current_class.clone();
        self.current_class = ClassType::Class;

        self.declare(&class.name);
        self.define(&class.name);

        if class.superclass.is_some()
            && class.name.lexeme == class.superclass.as_ref().unwrap().name.lexeme
        {
            error_token(
                &class.superclass.as_ref().unwrap().name,
                "A class can't inherit from itself.",
            );
        }

        if class.superclass.is_some() {
            self.current_class = ClassType::Subclass;
            self.resolve_expr(&Expr::Variable(class.superclass.clone().unwrap()));
        }

        if class.superclass.is_some() {
            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert("super".to_string(), true);
        }

        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert("this".to_string(), true);

        for method in &class.methods {
            let mut declaration = FunctionType::Method;
            if method.name.lexeme == "init" {
                declaration = FunctionType::Initializer;
            }
            self.resolve_function(method, declaration);
        }

        self.end_scope();

        if class.superclass.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;
    }

    fn visit_expression(&mut self, stmt: &stmt::Expression) -> Self::Output {
        self.resolve_expr(&stmt.expression);
    }

    fn visit_function(&mut self, function: &stmt::Function) -> Self::Output {
        self.declare(&function.name);
        self.define(&function.name);

        self.resolve_function(function, FunctionType::Function);
    }

    fn visit_if(&mut self, r#if: &stmt::If) -> Self::Output {
        self.resolve_expr(&r#if.condition);
        self.resolve_stmt(&r#if.then_branch);
        if r#if.else_branch.is_some() {
            self.resolve_stmt(r#if.else_branch.as_ref().unwrap());
        }
    }

    fn visit_print(&mut self, print: &stmt::Print) -> Self::Output {
        self.resolve_expr(&print.expression);
    }

    fn visit_return(&mut self, r#return: &stmt::Return) -> Self::Output {
        match self.current_function {
            FunctionType::None => {
                error_token(&r#return.keyword, "Can't return from top-level code.")
            }
            _ => {}
        }

        if r#return.value.is_some() {
            match self.current_function {
                FunctionType::Initializer => error_token(
                    &r#return.keyword,
                    "Can't return a value from an initializer.",
                ),
                _ => {}
            }
            self.resolve_expr(r#return.value.as_ref().unwrap());
        }
    }

    fn visit_var(&mut self, var: &stmt::Var) -> Self::Output {
        self.declare(&var.name);
        if var.initializer.is_some() {
            self.resolve_expr(var.initializer.as_ref().unwrap());
        }
        self.define(&var.name);
    }

    fn visit_while(&mut self, r#while: &stmt::While) -> Self::Output {
        self.resolve_expr(&r#while.condition);
        self.resolve_stmt(&r#while.body);
    }
}

impl expr::Visitor for Resolver {
    type Output = ();

    fn visit_assign(&mut self, assign: &expr::Assign) -> Self::Output {
        self.resolve_expr(&assign.value);
        self.resolve_local(&assign.name);
    }

    fn visit_binary(&mut self, binary: &expr::Binary) -> Self::Output {
        self.resolve_expr(&binary.left);
        self.resolve_expr(&binary.right);
    }

    fn visit_call(&mut self, call: &expr::Call) -> Self::Output {
        self.resolve_expr(&call.callee);

        for argument in &call.arguments {
            self.resolve_expr(argument);
        }
    }

    fn visit_get(&mut self, get: &expr::Get) -> Self::Output {
        self.resolve_expr(&get.object);
    }

    fn visit_grouping(&mut self, grouping: &expr::Grouping) -> Self::Output {
        self.resolve_expr(&grouping.expression);
    }

    fn visit_literal(&mut self, _: &expr::Literal) -> Self::Output {}

    fn visit_logical(&mut self, logical: &expr::Logical) -> Self::Output {
        self.resolve_expr(&logical.left);
        self.resolve_expr(&logical.right);
    }

    fn visit_set(&mut self, set: &expr::Set) -> Self::Output {
        self.resolve_expr(&set.value);
        self.resolve_expr(&set.object);
    }

    fn visit_super(&mut self, sup: &expr::Super) -> Self::Output {
        match self.current_class {
            ClassType::None => error_token(&sup.keyword, "can't use 'super' outside of a class."),
            ClassType::Subclass => {}
            _ => error_token(
                &sup.keyword,
                "Can't use 'super' in a class with no superclass.",
            ),
        }

        self.resolve_local(&sup.keyword);
    }

    fn visit_this(&mut self, this: &expr::This) -> Self::Output {
        match self.current_class {
            ClassType::None => {
                error_token(&this.keyword, "Can't use 'this' outside of a class.");
                return;
            }
            _ => {}
        }

        self.resolve_local(&this.keyword);
    }

    fn visit_unary(&mut self, unary: &expr::Unary) -> Self::Output {
        self.resolve_expr(&unary.right);
    }

    fn visit_variable(&mut self, variable: &expr::Variable) -> Self::Output {
        let scope_option = self.scopes.last();
        if scope_option.is_none() {
            return;
        }

        let scope = scope_option.unwrap();
        if scope
            .get(&variable.name.lexeme)
            .is_some_and(|v| v == &false)
        {
            error_token(
                &variable.name,
                "Can't read local variable in its own initializer",
            );
        }

        self.resolve_local(&variable.name);
    }
}
