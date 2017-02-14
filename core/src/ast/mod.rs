mod ident;
mod variable;
mod operator;
mod item;

use std::ops;

pub use ast::ident::*;
pub use ast::variable::*;
pub use ast::operator::*;
pub use ast::item::{Item, Value};

pub type Index = usize;

#[derive(Debug, PartialEq)]
pub struct Node<'src> {
    pub start: usize,
    pub end: usize,
    pub item: Item<'src>,
    pub next: Option<Index>,
}

pub struct Store<'src>(Vec<Node<'src>>);

pub struct List<'store, 'src: 'store> {
    next: Option<Index>,
    store: &'store Store<'src>,
}

pub struct Program<'src> {
    pub source: &'src str,
    pub root: Option<Index>,
    pub items: Store<'src>,
}

impl<'src> Node<'src> {
    #[inline]
    pub fn new(start: usize, end: usize, item: Item<'src>) -> Self {
        Node {
            start: start,
            end: end,
            item: item,
            next: None
        }
    }
}

impl<'src> Store<'src> {
    #[inline]
    pub fn new() -> Self {
        Store(Vec::with_capacity(128))
    }

    #[inline]
    pub fn insert(&mut self, node: Node<'src>) -> usize {
        let index = self.len();
        self.0.push(node);
        index
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn list<'store>(&'store self, from: Index) -> List<'store, 'src> {
        List {
            store: &self,
            next: if self.len() > from { Some(from) } else { None },
        }
    }
}

impl<'store, 'src: 'store> Program<'src> {
    #[inline]
    pub fn statements(&'src self) -> List<'store, 'src> {
        List {
            store: &self.items,
            next: self.root,
        }
    }
}

impl<'src> ops::Index<usize> for Store<'src> {
    type Output = Node<'src>;

    #[inline]
    fn index(&self, index: usize) -> &Node<'src> {
        &self.0[index]
    }
}

impl<'src> ops::IndexMut<usize> for Store<'src> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Node<'src> {
        &mut self.0[index]
    }
}

impl<'store, 'src: 'store> Iterator for List<'store, 'src> {
    type Item = &'store Item<'src>;

    #[inline]
    fn next(&mut self) -> Option<&'store Item<'src>> {
        let next = self.next;

        next.map(|id| {
            let node = &self.store[id];
            self.next = node.next;
            &node.item
        })
    }
}
