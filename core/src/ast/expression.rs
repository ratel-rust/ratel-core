use ast::Ident;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression {
    Void,
    This,
    Identifier(Ident),
}
