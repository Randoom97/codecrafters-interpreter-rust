use crate::{expr::Expr, token::Token};

pub trait Visitor {
    type Output;

    fn visit_block(&mut self, block: &Block) -> Self::Output;
    fn visit_expression(&mut self, stmt: &Expression) -> Self::Output;
    fn visit_if(&mut self, r#if: &If) -> Self::Output;
    fn visit_print(&mut self, print: &Print) -> Self::Output;
    fn visit_var(&mut self, var: &Var) -> Self::Output;
    fn visit_while(&mut self, r#while: &While) -> Self::Output;
}

pub enum Stmt {
    Block(Block),
    Expression(Expression),
    If(If),
    Print(Print),
    Var(Var),
    While(While),
}

impl Stmt {
    pub fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        return match self {
            Stmt::Block(block) => visitor.visit_block(block),
            Stmt::Expression(expression) => visitor.visit_expression(expression),
            Stmt::If(r#if) => visitor.visit_if(r#if),
            Stmt::Print(print) => visitor.visit_print(print),
            Stmt::Var(var) => visitor.visit_var(var),
            Stmt::While(r#while) => visitor.visit_while(r#while),
        };
    }
}

pub struct Block {
    pub statements: Vec<Stmt>,
}

impl Block {
    pub fn new(statements: Vec<Stmt>) -> Block {
        Block { statements }
    }
}

pub struct Expression {
    pub expression: Box<Expr>,
}

impl Expression {
    pub fn new(expression: Expr) -> Expression {
        Expression {
            expression: Box::new(expression),
        }
    }
}

pub struct If {
    pub condition: Box<Expr>,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

impl If {
    pub fn new(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> If {
        If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(|eb| Box::new(eb)),
        }
    }
}

pub struct Print {
    pub expression: Box<Expr>,
}

impl Print {
    pub fn new(expression: Expr) -> Print {
        Print {
            expression: Box::new(expression),
        }
    }
}

pub struct Var {
    pub name: Token,
    pub initializer: Option<Box<Expr>>,
}

impl Var {
    pub fn new(name: Token, initializer: Option<Expr>) -> Var {
        Var {
            name,
            initializer: initializer.map(|i| Box::new(i)),
        }
    }
}

pub struct While {
    pub condition: Box<Expr>,
    pub body: Box<Stmt>,
}

impl While {
    pub fn new(condition: Expr, body: Stmt) -> While {
        While {
            condition: Box::new(condition),
            body: Box::new(body),
        }
    }
}
