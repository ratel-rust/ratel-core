use ast::Expression;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Statement {
    Expression(Expression),
}
