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
}

pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
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

    fn visit_grouping(&mut self, grouping: &expr::Grouping) -> Self::Output {
        self.resolve_expr(&grouping.expression);
    }

    fn visit_literal(&mut self, _: &expr::Literal) -> Self::Output {}

    fn visit_logical(&mut self, logical: &expr::Logical) -> Self::Output {
        self.resolve_expr(&logical.left);
        self.resolve_expr(&logical.right);
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
