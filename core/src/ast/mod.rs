mod ident;
mod variable;
mod operator;

pub struct Expression;
pub struct Statement;

pub use ast::ident::*;
pub use ast::variable::*;
pub use ast::operator::*;

pub struct Index(usize);

pub struct Node<T> {
    begin: usize,
    end: usize,
    value: T,
    next: Option<Index>,
}

pub struct Program<'src> {
    pub source: &'src str,
    pub expressions: Vec<Node<Expression>>,
    pub statements: Vec<Node<Statement>>,
}
