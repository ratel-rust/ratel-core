use toolshed::Arena;
use ast::{Node, Loc, List, ListBuilder};

pub mod es2015;
pub mod es2016;

// mod scope;

// use self::scope::Scope;

pub struct Transformer<'ast> {
    pub arena: &'ast Arena,
    // _scope: Scope<'ast>,
}

impl<'ast> Transformer<'ast> {
    #[inline]
    pub fn alloc<T, I>(&self, item: I) -> Node<'ast, T> where
        T: Copy + 'ast,
        I: Into<T>,
    {
        Node::new(self.arena.alloc(Loc::new(0, 0, item.into())))
    }

    #[inline]
    pub fn alloc_as_loc<T, I, L>(&self, loc: Node<'ast, L>, item: I) -> Node<'ast, T> where
        T: Copy + 'ast,
        I: Into<T>,
    {
        Node::new(self.arena.alloc(Loc::new(loc.start, loc.end, item.into())))
    }

    #[inline]
    pub fn list<T, I>(&mut self, source: I) -> List<'ast, T> where
        T: 'ast + Copy,
        I: AsRef<[Node<'ast, T>]>
    {
        let mut iter = source.as_ref().into_iter();

        let mut builder = match iter.next() {
            Some(item) => ListBuilder::new(self.arena, *item),
            None       => return List::empty(),
        };

        for item in iter {
            builder.push(*item);
        }

        builder.into_list()
    }

    #[inline]
    pub fn swap<T, I>(&self, ptr: &Node<'ast, T>, item: I) where
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

// pub fn transform<'ast>(module: &mut Module, settings: Settings) {
//     let arena = module.arena();
//     let body = module.body();

//     let transformer = Transformer {
//         arena,
//         settings,
//         scope: Scope::new(arena),
//     };

//     transformer.transform(body);
// }
