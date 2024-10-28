use crate::token::{LiteralValue, Token};

pub trait Visitor {
    type Output;

    fn visit_assign(&mut self, assign: &Assign) -> Self::Output;
    fn visit_binary(&mut self, binary: &Binary) -> Self::Output;
    fn visit_grouping(&mut self, grouping: &Grouping) -> Self::Output;
    fn visit_literal(&mut self, literal: &Literal) -> Self::Output;
    fn visit_logical(&mut self, logical: &Logical) -> Self::Output;
    fn visit_unary(&mut self, unary: &Unary) -> Self::Output;
    fn visit_variable(&mut self, variable: &Variable) -> Self::Output;
}

pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Unary(Unary),
    Variable(Variable),
}

impl Expr {
    pub fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        return match self {
            Expr::Assign(assign) => visitor.visit_assign(assign),
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Logical(logical) => visitor.visit_logical(logical),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Variable(variable) => visitor.visit_variable(variable),
        };
    }
}

pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

impl Assign {
    pub fn new(name: Token, value: Expr) -> Assign {
        Assign {
            name,
            value: Box::new(value),
        }
    }
}

pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Binary {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Binary {
        Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

pub struct Grouping {
    pub expression: Box<Expr>,
}

impl Grouping {
    pub fn new(expression: Expr) -> Grouping {
        Grouping {
            expression: Box::new(expression),
        }
    }
}

pub struct Literal {
    pub value: Option<LiteralValue>,
}

impl Literal {
    pub fn new(value: Option<LiteralValue>) -> Literal {
        Literal { value }
    }
}

pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Logical {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Logical {
        Logical {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Unary {
    pub fn new(operator: Token, right: Expr) -> Unary {
        Unary {
            operator,
            right: Box::new(right),
        }
    }
}

pub struct Variable {
    pub name: Token,
}

impl Variable {
    pub fn new(name: Token) -> Variable {
        Variable { name }
    }
}
