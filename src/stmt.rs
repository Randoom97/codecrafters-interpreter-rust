use crate::{expr::Expr, token::Token};

pub trait Visitor {
    type Output;

    fn visit_block(&mut self, block: &Block) -> Self::Output;
    fn visit_expression(&mut self, stmt: &Expression) -> Self::Output;
    fn visit_print(&mut self, print: &Print) -> Self::Output;
    fn visit_var(&mut self, var: &Var) -> Self::Output;
}

pub enum Stmt {
    Block(Block),
    Expression(Expression),
    Print(Print),
    Var(Var),
}

impl Stmt {
    pub fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        return match self {
            Stmt::Block(block) => visitor.visit_block(block),
            Stmt::Expression(expression) => visitor.visit_expression(expression),
            Stmt::Print(print) => visitor.visit_print(print),
            Stmt::Var(var) => visitor.visit_var(var),
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
