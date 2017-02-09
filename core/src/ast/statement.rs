use ast::ExpressionId;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Statement {
    Expression(ExpressionId),
}
