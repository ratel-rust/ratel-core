#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate ratel;
extern crate ratel_visitor;
extern crate toolshed;

use toolshed::Arena;
use toolshed::list::ListBuilder;
use ratel::ast::{Loc, Node, NodeList};
use ratel::Module;
use ratel_visitor::{ScopeKind, Visitable};

pub mod es2015;
// pub mod es2016;

pub mod scope;

use self::scope::Scope;

#[derive(Copy, Clone)]
pub struct TransformerCtxt<'ast> {
    pub arena: &'ast Arena,
    pub scope: &'ast Scope<'ast>,
}

impl<'ast> TransformerCtxt<'ast> {
    #[inline]
    pub fn alloc<T, I>(&self, item: I) -> Node<'ast, T> where
        T: Copy,
        I: Into<T>,
    {
        Node::new(self.arena.alloc(Loc::new(0, 0, item.into())))
    }

    #[inline]
    pub fn alloc_as_loc<T, I, L>(&self, loc: &Node<'ast, L>, item: I) -> Node<'ast, T> where
        T: Copy + 'ast,
        I: Into<T>,
    {
        Node::new(self.arena.alloc(Loc::new(loc.start, loc.end, item.into())))
    }

    #[inline]
    pub fn list<T, I>(&mut self, source: I) -> NodeList<'ast, T> where
        T: 'ast + Copy,
        I: AsRef<[Node<'ast, T>]>
    {
        let mut iter = source.as_ref().iter();

        let builder = match iter.next() {
            Some(item) => ListBuilder::new(self.arena, *item),
            None       => return NodeList::empty(),
        };

        for item in iter {
            builder.push(self.arena, *item);
        }

        builder.as_list()
    }

    #[inline]
    pub fn swap<T, I>(&self, ptr: Node<'ast, T>, item: I) where
        T: Copy + 'ast,
        I: Into<T>,
    {
        let new = self.arena.alloc(Loc {
            start: ptr.start,
            end: ptr.end,
            item: item.into()
        });

        ptr.set(new);
    }
}

pub fn transform<'ast>(module: &'ast mut Module<'ast>) {
    let arena = module.arena();

    let ctx = TransformerCtxt {
        arena,
        scope: arena.alloc(Scope::new(ScopeKind::Function, None)),
    };

    module.visit_with(&mut es2015::PresetES2015::new(ctx));
}
