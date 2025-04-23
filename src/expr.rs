use crate::token::{LiteralValue, Token};

pub trait Visitor {
    type Output;

    fn visit_assign(&mut self, assign: &Assign) -> Self::Output;
    fn visit_binary(&mut self, binary: &Binary) -> Self::Output;
    fn visit_call(&mut self, call: &Call) -> Self::Output;
    fn visit_get(&mut self, get: &Get) -> Self::Output;
    fn visit_grouping(&mut self, grouping: &Grouping) -> Self::Output;
    fn visit_literal(&mut self, literal: &Literal) -> Self::Output;
    fn visit_logical(&mut self, logical: &Logical) -> Self::Output;
    fn visit_set(&mut self, set: &Set) -> Self::Output;
    fn visit_this(&mut self, this: &This) -> Self::Output;
    fn visit_unary(&mut self, unary: &Unary) -> Self::Output;
    fn visit_variable(&mut self, variable: &Variable) -> Self::Output;
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Call(Call),
    Get(Get),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Set(Set),
    This(This),
    Unary(Unary),
    Variable(Variable),
}

impl Expr {
    pub fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        return match self {
            Expr::Assign(assign) => visitor.visit_assign(assign),
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Call(call) => visitor.visit_call(call),
            Expr::Get(get) => visitor.visit_get(get),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Logical(logical) => visitor.visit_logical(logical),
            Expr::Set(set) => visitor.visit_set(set),
            Expr::This(this) => visitor.visit_this(this),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Variable(variable) => visitor.visit_variable(variable),
        };
    }
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

impl Call {
    pub fn new(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Call {
        Call {
            callee: Box::new(callee),
            paren,
            arguments,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Get {
    pub object: Box<Expr>,
    pub name: Token,
}

impl Get {
    pub fn new(object: Expr, name: Token) -> Get {
        Get {
            object: Box::new(object),
            name,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct Literal {
    pub value: Option<LiteralValue>,
}

impl Literal {
    pub fn new(value: Option<LiteralValue>) -> Literal {
        Literal { value }
    }
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct Set {
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}

impl Set {
    pub fn new(object: Expr, name: Token, value: Expr) -> Set {
        Set {
            object: Box::new(object),
            name,
            value: Box::new(value),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct This {
    pub keyword: Token,
}

impl This {
    pub fn new(keyword: Token) -> This {
        This { keyword }
    }
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub name: Token,
}

impl Variable {
    pub fn new(name: Token) -> Variable {
        Variable { name }
    }
}
