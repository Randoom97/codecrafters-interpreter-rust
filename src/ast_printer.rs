use crate::expr::{self, Expr};

pub struct AstPrinter {}

impl AstPrinter {
    pub fn new() -> AstPrinter {
        return AstPrinter {};
    }

    pub fn print(&mut self, expr: &Expr) -> String {
        return expr.accept(self);
    }

    fn parenthesize(&mut self, name: &String, exprs: &Vec<&Box<Expr>>) -> String {
        let mut string = format!("({}", name);
        for expr in exprs {
            string += " ";
            string += expr.accept(self).as_str();
        }
        string += ")";
        return string;
    }
}

impl expr::Visitor for AstPrinter {
    type Output = String;

    fn visit_assign(&mut self, assign: &expr::Assign) -> Self::Output {
        return self.parenthesize(&assign.name.lexeme, &vec![&assign.value]);
    }

    fn visit_binary(&mut self, binary: &crate::expr::Binary) -> String {
        return self.parenthesize(&binary.operator.lexeme, &vec![&binary.left, &binary.right]);
    }

    fn visit_grouping(&mut self, grouping: &crate::expr::Grouping) -> String {
        return self.parenthesize(&"group".to_string(), &vec![&grouping.expression]);
    }

    fn visit_literal(&mut self, literal: &crate::expr::Literal) -> String {
        if literal.value.is_none() {
            return "nil".to_string();
        }
        return literal.value.as_ref().unwrap().to_string();
    }

    fn visit_unary(&mut self, unary: &crate::expr::Unary) -> String {
        return self.parenthesize(&unary.operator.lexeme, &vec![&unary.right]);
    }

    fn visit_variable(&mut self, variable: &expr::Variable) -> Self::Output {
        return variable.name.lexeme.clone();
    }
}
