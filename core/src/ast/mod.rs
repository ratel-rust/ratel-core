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

pub struct Nodes<'store, 'src: 'store> {
    next: Option<Index>,
    store: &'store Store<'src>,
}

pub struct Items<'store, 'src: 'store>(Nodes<'store, 'src>);

pub struct Program<'src> {
    pub source: &'src str,
    pub root: Option<Index>,
    pub store: Store<'src>,
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
        Store(Vec::with_capacity(256))
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
    pub fn nodes<'store, I>(&'store self, from: I) -> Nodes<'store, 'src> where
        I: Into<Option<Index>>
    {
        Nodes {
            store: &self,
            next: from.into(),
        }
    }
}

impl<'store, 'src: 'store> Program<'src> {
    #[inline]
    pub fn statements(&'src self) -> Nodes<'store, 'src> {
        self.store.nodes(self.root)
    }
}

impl<'src> ops::Index<usize> for Program<'src> {
    type Output = Item<'src>;

    #[inline]
    fn index(&self, index: usize) -> &Item<'src> {
        &self.store[index].item
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

impl<'store, 'src: 'store> Nodes<'store, 'src> {
    #[inline]
    pub fn items(self) -> Items<'store, 'src> {
        Items(self)
    }
}

impl<'store, 'src: 'store> Iterator for Nodes<'store, 'src> {
    type Item = &'store Node<'src>;

    #[inline]
    fn next(&mut self) -> Option<&'store Node<'src>> {
        let next = self.next;

        next.map(|id| {
            let node = &self.store[id];
            self.next = node.next;
            node
        })
    }
}

impl<'store, 'src: 'store> Iterator for Items<'store, 'src> {
    type Item = &'store Item<'src>;

    #[inline]
    fn next(&mut self) -> Option<&'store Item<'src>> {
        self.0.next().map(|node| &node.item)
    }
}
