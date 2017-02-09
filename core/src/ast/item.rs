use ast::{Index, Ident, OperatorKind};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Item {
    ExpressionStatement(Index),
    Identifier(Ident),
    This,
    BinaryExpr {
        parenthesized: bool,
        operator: OperatorKind,
        left: Index,
        right: Index,
    },
    PostfixExpr {
        operator: OperatorKind,
        operand: Index,
    }
}
