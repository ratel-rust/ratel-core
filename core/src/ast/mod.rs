#[macro_use]
mod list;
mod variable;
mod operator;
mod types;
mod function;
mod literal;
pub mod ptr;
pub mod expression;
pub mod statement;

use std::ops::Deref;

pub use ast::variable::*;
pub use ast::operator::*;
pub use ast::ptr::Ptr;
pub use ast::types::{Type, Primitive};
pub use ast::expression::{Expression, ObjectMember, Property};
pub use ast::statement::{Statement, Declarator, DeclaratorId, BlockStatement};
pub use ast::function::{Function, Class, ClassMember};
pub use ast::function::{Name, OptionalName, MandatoryName, Parameter, ParameterKey};
pub use ast::literal::Literal;
pub use ast::list::{RawList, List, ListIter, ListBuilder, EmptyListBuilder};

// Handful of useful aliases
pub type BlockPtr<'ast> = Ptr<'ast, Loc<BlockStatement<'ast>>>;
pub type PropertyPtr<'ast> = Ptr<'ast, Loc<Property<'ast>>>;
pub type ParameterPtr<'ast> = Ptr<'ast, Loc<Parameter<'ast>>>;
pub type ParameterList<'ast> = List<'ast, Loc<Parameter<'ast>>>;
pub type TypePtr<'ast> = List<'ast, Loc<Type<'ast>>>;
pub type TypeList<'ast> = List<'ast, Loc<Type<'ast>>>;
pub type ExpressionPtr<'ast> = Ptr<'ast, Loc<Expression<'ast>>>;
pub type ExpressionList<'ast> = List<'ast, Loc<Expression<'ast>>>;
pub type StatementPtr<'ast> = Ptr<'ast, Loc<Statement<'ast>>>;
pub type StatementList<'ast> = List<'ast, Loc<Statement<'ast>>>;
pub type IdentifierPtr<'ast> = Ptr<'ast, Loc<&'ast str>>;
pub type IdentifierList<'ast> = List<'ast, Loc<&'ast str>>;

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
    pub body: List<'ast, Loc<Statement<'ast>>>,
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
    pub fn statements(&'ast self) -> &'ast List<'ast, Loc<Statement<'ast>>> {
        &self.body
    }
}
