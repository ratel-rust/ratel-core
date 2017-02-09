use ast::{ExpressionId, Ident, OperatorKind};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression {
    Void,
    This,
    Identifier(Ident),
    Binary {
        parenthesized: bool,
        operator: OperatorKind,
        left: ExpressionId,
        right: ExpressionId,
    },
    Postfix {
        operator: OperatorKind,
        operand: ExpressionId,
    }
}
