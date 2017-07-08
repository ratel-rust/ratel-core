use std::cell::Cell;
use ast::{Loc, List, Value, OperatorKind, Function, Class, OptionalName};
use ast::{PropertyPtr, ExpressionPtr, ExpressionList, StatementPtr, StatementList, IdentifierPtr, IdentifierList};

#[derive(Debug, PartialEq, Clone)]
pub enum Property<'ast> {
    Computed(ExpressionPtr<'ast>),
    Literal(&'ast str),
    Binary(u64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectMember<'ast> {
    Shorthand(&'ast str),
    Value {
        key: PropertyPtr<'ast>,
        value: ExpressionPtr<'ast>,
    },
    Method {
        key: PropertyPtr<'ast>,
        params: IdentifierList<'ast>,
        body: StatementList<'ast>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<'ast> {
    Error,
    This,
    Identifier(&'ast str),
    Value(Value<'ast>),
    Sequence {
        body: ExpressionList<'ast>
    },
    Array {
        body: ExpressionList<'ast>
    },
    Member {
        object: ExpressionPtr<'ast>,
        property: IdentifierPtr<'ast>,
    },
    ComputedMember {
        object: ExpressionPtr<'ast>,
        property: ExpressionPtr<'ast>,
    },
    Call {
        callee: ExpressionPtr<'ast>,
        arguments: ExpressionList<'ast>,
    },
    Binary {
        parenthesized: Cell<bool>,
        operator: OperatorKind,
        left: ExpressionPtr<'ast>,
        right: ExpressionPtr<'ast>,
    },
    Prefix {
        operator: OperatorKind,
        operand: ExpressionPtr<'ast>,
    },
    Postfix {
        operator: OperatorKind,
        operand: ExpressionPtr<'ast>,
    },
    Conditional {
        test: ExpressionPtr<'ast>,
        consequent: ExpressionPtr<'ast>,
        alternate: ExpressionPtr<'ast>,
    },
    Arrow {
        params: IdentifierList<'ast>,
        body: StatementPtr<'ast>,
    },
    Object {
        body: List<'ast, Loc<ObjectMember<'ast>>>,
    },
    Function {
        function: Function<'ast, OptionalName<'ast>>,
    },
    Class {
        class: Class<'ast, OptionalName<'ast>>,
    },
}

impl<'ast> Expression<'ast> {
    #[inline]
    pub fn at(self, start: u32, end: u32) -> Loc<Expression<'ast>> {
        Loc::new(start, end, self)
    }

    #[inline]
    pub fn parenthesize(&self) {
        if let Expression::Binary { ref parenthesized, .. } = *self {
            parenthesized.set(true);
        }
    }

    #[inline]
    pub fn binding_power(&self) -> u8 {
        use self::Expression::*;

        match *self {
            Member { .. } | Arrow { .. } => 18,

            Call { .. } => 17,

            Prefix { .. } => 15,

            Binary { ref operator, .. }  |
            Postfix { ref operator, .. } => operator.binding_power(),

            Conditional { .. } => 4,

            _  => 100,
        }
    }

    #[inline]
    pub fn needs_parens(&self, bp: u8) -> bool {
        match *self {
            Expression::Binary {
                ref parenthesized,
                ref operator,
                ..
            } => parenthesized.get() && bp >= operator.binding_power(),
            _ => false
        }
    }

    #[inline]
    pub fn is_allowed_as_bare_statement(&self) -> bool {
        use self::Expression::*;

        match *self {
            Object { .. }   |
            Function { .. } |
            Class { .. }    => false,
            _               => true,
        }
    }
}
