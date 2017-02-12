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
pub struct Node {
    pub start: usize,
    pub end: usize,
    pub item: Item,
    pub next: Option<Index>,
}

pub struct Store(Vec<Node>);

pub struct List<'store> {
    next: Option<Index>,
    store: &'store Store,
}

pub struct Program<'src> {
    pub source: &'src str,
    pub root: Option<Index>,
    pub items: Store,
}

impl Node {
    #[inline]
    pub fn new(start: usize, end: usize, item: Item) -> Self {
        Node {
            start: start,
            end: end,
            item: item,
            next: None
        }
    }
}

impl Store {
    #[inline]
    pub fn new() -> Self {
        Store(Vec::with_capacity(128))
    }

    #[inline]
    pub fn insert(&mut self, node: Node) -> usize {
        let index = self.len();
        self.0.push(node);
        index
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn list(&self, from: Index) -> List {
        List {
            store: &self,
            next: if self.len() > from { Some(from) } else { None },
        }
    }
}

impl<'store, 'src: 'store> Program<'src> {
    #[inline]
    pub fn statements(&'src self) -> List<'store> {
        List {
            store: &self.items,
            next: self.root,
        }
    }
}

impl ops::Index<usize> for Store {
    type Output = Node;

    #[inline]
    fn index(&self, index: usize) -> &Node {
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for Store {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Node {
        &mut self.0[index]
    }
}

impl<'store> Iterator for List<'store> {
    type Item = &'store Item;

    #[inline]
    fn next(&mut self) -> Option<&'store Item> {
        let next = self.next;

        next.map(|id| {
            let node = &self.store[id];
            self.next = node.next;
            &node.item
        })
    }
}
