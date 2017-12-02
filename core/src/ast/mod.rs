#[macro_use]
mod variable;
mod operator;
// mod types;
mod function;
mod literal;
pub mod node;
pub mod expression;
pub mod statement;

use toolshed::list::List;
use std::ops::Deref;

pub use ast::variable::*;
pub use ast::operator::*;
pub use ast::node::Node;
// pub use ast::types::{Type, Primitive};
pub use ast::expression::{Expression, Property, PropertyKey};
pub use ast::statement::{Statement, Declarator, BlockStatement};
pub use ast::function::{Function, Class, ClassMember, Method, MethodKind};
pub use ast::function::{Name, EmptyName, OptionalName, MandatoryName};
pub use ast::literal::Literal;


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Block<'ast, T: 'ast> {
    pub body: NodeList<'ast, T>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Pattern<'ast> {
    /// Only used inside ArrayPattern
    Void,
    Identifier(Identifier<'ast>),
    ObjectPattern {
        properties: NodeList<'ast, Property<'ast>>
    },
    ArrayPattern {
        elements: NodeList<'ast, Pattern<'ast>>
    },
    RestElement {
        argument: IdentifierNode<'ast>
    },
    AssignmentPattern {
        left: Node<'ast, Pattern<'ast>>,
        right: ExpressionNode<'ast>,
    }
}

// Handful of useful aliases
pub type Identifier<'ast> = &'ast str;
pub type NodeList<'ast, T> = List<'ast, Node<'ast, T>>;
pub type BlockNode<'ast, T> = Node<'ast, Block<'ast, T>>;
pub type PatternList<'ast> = NodeList<'ast, Pattern<'ast>>;
pub type PropertyNode<'ast> = Node<'ast, Property<'ast>>;
pub type ExpressionNode<'ast> = Node<'ast, Expression<'ast>>;
pub type ExpressionList<'ast> = NodeList<'ast, Expression<'ast>>;
pub type StatementNode<'ast> = Node<'ast, Statement<'ast>>;
pub type StatementList<'ast> = NodeList<'ast, Statement<'ast>>;
pub type IdentifierNode<'ast> = Node<'ast, &'ast str>;
pub type IdentifierList<'ast> = NodeList<'ast, &'ast str>;
// pub type TypeNode<'ast> = NodeList<'ast, Type<'ast>>;
// pub type TypeList<'ast> = NodeList<'ast, Type<'ast>>;

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
    pub body: NodeList<'ast, Statement<'ast>>,
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
    pub fn statements(&'ast self) -> &'ast NodeList<'ast, Statement<'ast>> {
        &self.body
    }
}
