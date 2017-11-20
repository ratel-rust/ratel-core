#[macro_use]
mod list;
mod variable;
mod operator;
// mod types;
mod function;
mod literal;
pub mod ptr;
pub mod expression;
pub mod statement;

use std::ops::Deref;

pub use ast::variable::*;
pub use ast::operator::*;
pub use ast::ptr::Ptr;
// pub use ast::types::{Type, Primitive};
pub use ast::expression::{Expression, Property, PropertyKey};
pub use ast::statement::{Statement, Declarator, BlockStatement};
pub use ast::function::{Function, Class, ClassMember, MethodKind};
pub use ast::function::{Name, EmptyName, OptionalName, MandatoryName};
pub use ast::literal::Literal;
pub use ast::list::{RawList, List, ListIter, ListBuilder, EmptyListBuilder};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Block<'ast, T: 'ast> {
    pub body: List<'ast, T>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Pattern<'ast> {
    /// Only used inside ArrayPattern
    Void,
    Identifier(&'ast str),
    ObjectPattern {
        properties: List<'ast, Property<'ast>>
    },
    ArrayPattern {
        elements: List<'ast, Pattern<'ast>>
    },
    RestElement {
        argument: IdentifierPtr<'ast>
    },
    AssignmentPattern {
        left: Ptr<'ast, Pattern<'ast>>,
        right: ExpressionPtr<'ast>,
    }
}

// Handful of useful aliases
pub type BlockPtr<'ast, T> = Ptr<'ast, Block<'ast, T>>;
pub type PropertyPtr<'ast> = Ptr<'ast, Property<'ast>>;
pub type ExpressionPtr<'ast> = Ptr<'ast, Expression<'ast>>;
pub type ExpressionList<'ast> = List<'ast, Expression<'ast>>;
pub type StatementPtr<'ast> = Ptr<'ast, Statement<'ast>>;
pub type StatementList<'ast> = List<'ast, Statement<'ast>>;
pub type IdentifierPtr<'ast> = Ptr<'ast, &'ast str>;
pub type IdentifierList<'ast> = List<'ast, &'ast str>;
// pub type TypePtr<'ast> = List<'ast, Type<'ast>>;
// pub type TypeList<'ast> = List<'ast, Type<'ast>>;

#[derive(Debug, Clone)]
pub struct Loc<T> {
    pub start: u32,
    pub end: u32,
    pub item: T,
}

impl<T: Copy> Copy for Loc<T> {}

impl<T> Deref for Loc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.item
    }
}

pub struct Program<'ast> {
    pub source: &'ast str,
    pub body: List<'ast, Statement<'ast>>,
}

impl<T> Loc<T> {
    #[inline]
    pub fn new(start: u32, end: u32, item: T) -> Self {
        Loc {
            start,
            end,
            item,
        }
    }
}

impl<T: PartialEq> PartialEq for Loc<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.item.eq(&other.item)
    }
}

impl<'ast> Program<'ast> {
    #[inline]
    pub fn statements(&'ast self) -> &'ast List<'ast, Statement<'ast>> {
        &self.body
    }
}
