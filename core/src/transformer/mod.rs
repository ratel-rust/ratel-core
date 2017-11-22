use ast::{ExpressionPtr, StatementPtr};

pub mod expression;

mod scope;
// mod settings;
// mod statement;
// mod expression;

use arena::Arena;
// use module::Module;
use ast::{Ptr, Loc};

pub use self::scope::Scope;

pub struct Transformer<'ast> {
    arena: &'ast Arena,
    _scope: Scope<'ast>,
}

impl<'ast> Transformer<'ast> {
    #[inline]
    pub fn alloc<T, I>(&self, item: I) -> Ptr<'ast, T> where
        T: Copy + 'ast,
        I: Into<T>,
    {
        Ptr::new(self.arena.alloc(Loc::new(0, 0, item.into())))
    }

    #[inline]
    pub fn alloc_as_loc<T, I, L>(&self, loc: Ptr<'ast, L>, item: I) -> Ptr<'ast, T> where
        T: Copy + 'ast,
        I: Into<T>,
    {
        Ptr::new(self.arena.alloc(Loc::new(loc.start, loc.end, item.into())))
    }

    #[inline]
    pub fn swap<T, I>(&self, ptr: &Ptr<'ast, T>, item: I) where
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
