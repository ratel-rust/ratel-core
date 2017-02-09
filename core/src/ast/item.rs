use ast::{Node, Index, Ident, OperatorKind};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Item {
    // Identifiers
    Identifier(Ident),
    This,

    // Expressions
    ArrayExpr(Index),
    SequenceExpr(Index),
    MemberExpr {
        object: Index,
        property: Index,
    },
    CallExpr {
        callee: Index,
        arguments: Option<Index>,
    },
    BinaryExpr {
        parenthesized: bool,
        operator: OperatorKind,
        left: Index,
        right: Index,
    },
    Prefix {
        operator: OperatorKind,
        operand: Index,
    },
    PostfixExpr {
        operator: OperatorKind,
        operand: Index,
    },
    ConditionalExpr {
        test: Index,
        consequent: Index,
        alternate: Index,
    },
    ArrowExpr {
        params: Option<Index>,
        body: Option<Index>,
    },
    FunctionExpr {
        name: Option<Ident>,
        params: Option<Index>,
        body: Option<Index>,
    },
    ClassExpr {
        name: Option<Ident>,
        extends: Option<Ident>,
        body: Option<Index>,
    },

    // Statements
    ExpressionStatement(Index),
}

impl Item {
    #[inline]
    pub fn at(self, start: usize, end: usize) -> Node {
        Node::new(start, end, self)
    }
}
