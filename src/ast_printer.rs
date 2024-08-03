use crate::ast::{Expr, Visitor};

pub struct AstPrinter {}

impl AstPrinter {
    pub fn new() -> AstPrinter {
        return AstPrinter {};
    }

    pub fn print(&self, expr: &Expr) -> String {
        return expr.accept(self);
    }

    fn parenthesize(&self, name: &String, exprs: &Vec<&Box<Expr>>) -> String {
        let mut string = format!("({}", name);
        for expr in exprs {
            string += " ";
            string += expr.accept(self).as_str();
        }
        string += ")";
        return string;
    }
}

impl Visitor for AstPrinter {
    fn visit_binary(&self, binary: &crate::ast::Binary) -> String {
        return self.parenthesize(&binary.operator.lexeme, &vec![&binary.left, &binary.right]);
    }

    fn visit_grouping(&self, grouping: &crate::ast::Grouping) -> String {
        return self.parenthesize(&"group".to_string(), &vec![&grouping.expression]);
    }

    fn visit_literal(&self, literal: &crate::ast::Literal) -> String {
        if literal.value.is_none() {
            return "nil".to_string();
        }
        return literal.value.as_ref().unwrap().to_string();
    }

    fn visit_unary(&self, unary: &crate::ast::Unary) -> String {
        return self.parenthesize(&unary.operator.lexeme, &vec![&unary.right]);
    }
}
