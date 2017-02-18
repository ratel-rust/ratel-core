use ast::{Node, Index, Ident, OperatorKind, VariableDeclarationKind};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Value<'src> {
    Undefined,
    Null,
    True,
    False,
    Number(&'src str),
    Binary(u64),
    String(&'src str),
    RawQuasi(&'src str),
    RegEx {
         pattern: &'src str,
         flags: &'src str
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Item<'src> {
    // Identifiers
    Identifier(Ident<'src>),
    This,

    // Expressions
    ValueExpr(Value<'src>),
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
        name: Option<Ident<'src>>,
        params: Option<Index>,
        body: Option<Index>,
    },
    ObjectExpr {
        body: Option<Index>,
    },
    ClassExpr {
        name: Option<Ident<'src>>,
        extends: Option<Ident<'src>>,
        body: Option<Index>,
    },

    // Object
    ShorthandMember(Ident<'src>),
    ObjectMember {
        key: Ident<'src>,
        value: Index,
    },
    ComputedMember {
        key: Index,
        value: Index,
    },

    // Declaration
    VariableDeclarator {
        name: Index,
        value: Option<Index>,
    },

    // Statements
    EmptyStatement,
    ExpressionStatement(Index),
    DeclarationStatemenet {
        kind: VariableDeclarationKind,
        declarators: Index,
    },
    FunctionStatement {
        name: Ident<'src>,
        params: Option<Index>,
        body: Option<Index>,
    },
    ReturnStatement {
        value: Option<Index>,
    },
    BreakStatement {
        label: Option<Index>,
    },
    IfStatement {
        test: Index,
        consequent: Index,
        alternate: Option<Index>
    },
    WhileStatement {
        test: Index,
        body: Index,
    },
    DoStatement {
        test: Index,
        body: Index,
    },
    ForStatement {
        init: Option<Index>,
        test: Option<Index>,
        update: Option<Index>,
        body: Index
    },
    ForIn {
        left: Index,
        right: Index,
        body: Index
    },
    ForOf {
        left: Index,
        right: Index,
        body: Index
    },
    ThrowStatement {
        value: Index
    },
    TryStatement {
        body: Option<Index>,
        error: Ident<'src>,
        handler: Option<Index>
    }

}

impl<'src> Item<'src> {
    #[inline]
    pub fn at(self, start: usize, end: usize) -> Node<'src> {
        Node::new(start, end, self)
    }
}
