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
    RegEx(&'src str),
}

// Individual variants of this enum should not exceed 4 pointer-sized words.
// Due to alignment something like `Option<Indent<'src>>` is already 4 words,
// when possible things like identifiers should be stored as their own items.
//
// Also note that `OptIndex` is a single word, just as a regular `Index`, so
// doing optional fields this way is best.

/// `Item` contains all elements of JS syntax necessary to construct an AST.
/// This includes things like statements and expressions, as well as lower
/// level elements such as `Identifier`s.
///
/// All `Item`s are stored inside `Node` wrappers on a single `Vec` based store.
/// Recursion within the tree is achieved by referencing other `Node`s (and thus
/// `Item`s) by indices within that store. There are two kinds of indices:
///
/// * `Index` - alias to `usize`.
/// * `OptIndex` - Nullable index that's isomorphic to `usize`.
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
        name: OptIndex,
        params: OptIndex,
        body: OptIndex,
    },
    ObjectExpr {
        body: OptIndex,
    },
    ClassExpr {
        name: OptIndex,
        extends: OptIndex,
        body: OptIndex,
    },

    // Object
    ShorthandMember(Ident<'src>),
    ObjectMember {
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
        name: Index,
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
        error: Index,
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
