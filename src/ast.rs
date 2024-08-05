use crate::token::{LiteralValue, Token};

pub trait Visitor {
    type Output;

    fn visit_binary(&self, binary: &Binary) -> Self::Output;
    fn visit_grouping(&self, grouping: &Grouping) -> Self::Output;
    fn visit_literal(&self, literal: &Literal) -> Self::Output;
    fn visit_unary(&self, unary: &Unary) -> Self::Output;
}

pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

impl Expr {
    pub fn accept<T: Visitor>(&self, visitor: &T) -> T::Output {
        return match self {
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Unary(unary) => visitor.visit_unary(unary),
        };
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
