mod ident;
mod variable;
mod operator;
mod statement;
mod expression;

use std::ops::{Index, IndexMut};

pub use ast::ident::*;
pub use ast::variable::*;
pub use ast::operator::*;
pub use ast::expression::Expression;
pub use ast::statement::Statement;

pub type ExpressionId = usize;
pub type StatementId = usize;

#[derive(Debug, PartialEq)]
pub struct Node<T> {
    pub start: usize,
    pub end: usize,
    pub value: T,
    pub next: Option<usize>,
}

pub struct Store<T>(Vec<Node<T>>);

pub struct List<'store, T: 'store> {
    next: Option<usize>,
    store: &'store Store<T>,
}

pub struct Program<'src> {
    pub source: &'src str,
    pub expressions: Store<Expression>,
    pub statements: Store<Statement>,
}

impl<T> Node<T> {
    #[inline]
    pub fn new(start: usize, end: usize, val: T) -> Self {
        Node {
            start: start,
            end: end,
            value: val,
            next: None
        }
    }
}

impl<T> Store<T> {
    #[inline]
    pub fn new() -> Self {
        Store(Vec::new())
    }

    #[inline]
    pub fn insert(&mut self, start: usize, end: usize, val: T) -> usize {
        let index = self.len();
        self.0.push(Node::new(start, end, val));
        index
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn list(&self, from: usize) -> List<T> {
        List {
            store: &self,
            next: if self.len() > from { Some(from) } else { None },
        }
    }
}

impl<T> Index<usize> for Store<T> {
    type Output = Node<T>;

    #[inline]
    fn index(&self, index: usize) -> &Node<T> {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for Store<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Node<T> {
        &mut self.0[index]
    }
}

impl<'store, T> Iterator for List<'store, T> {
    type Item = &'store T;

    #[inline]
    fn next(&mut self) -> Option<&'store T> {
        let next = self.next;

        next.map(|id| {
            let node = &self.store[id];
            self.next = node.next;
            &node.value
        })
    }
}
