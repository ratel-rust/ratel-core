use ast::{Node, Index, OptIndex, Ident, OperatorKind, VariableDeclarationKind};
use error::Error;

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
        flags: &'src str,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Item<'src> {
    // Error
    Error(Error),

    // Identifiers
    Identifier(Ident<'src>),
    This,

    // Expressions
    ValueExpr(Value<'src>),
    ArrayExpr(OptIndex),
    SequenceExpr(Index),
    MemberExpr {
        object: Index,
        property: Index,
    },
    CallExpr {
        callee: Index,
        arguments: OptIndex,
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
        params: OptIndex,
        body: OptIndex,
    },
    FunctionExpr {
        name: Option<Ident<'src>>,
        params: OptIndex,
        body: OptIndex,
    },
    ObjectExpr {
        body: OptIndex,
    },
    ClassExpr {
        name: Option<Ident<'src>>,
        extends: Option<Ident<'src>>,
        body: OptIndex,
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
        value: OptIndex,
    },

    // Statements
    EmptyStatement,
    ExpressionStatement(Index),
    DeclarationStatement {
        kind: VariableDeclarationKind,
        declarators: Index,
    },
    FunctionStatement {
        name: Ident<'src>,
        params: OptIndex,
        body: OptIndex,
    },
    ReturnStatement {
        value: OptIndex,
    },
    BreakStatement {
        label: OptIndex,
    },
    IfStatement {
        test: Index,
        consequent: Index,
        alternate: OptIndex
    },
    WhileStatement {
        test: Index,
        body: Index,
    },
    DoStatement {
        body: Index,
        test: Index,
    },
    ForStatement {
        init: OptIndex,
        test: OptIndex,
        update: OptIndex,
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
        body: OptIndex,
        error: Ident<'src>,
        handler: OptIndex
    },
    BlockStatement {
        body: OptIndex
    }
}

impl<'src> Item<'src> {
    #[inline(always)]
    pub fn at(self, start: usize, end: usize) -> Node<'src> {
        Node::new(start, end, self)
    }

    #[inline(always)]
    pub fn identifier<I: Into<Ident<'src>>>(ident: I) -> Item<'src> {
        Item::Identifier(ident.into())
    }
}
