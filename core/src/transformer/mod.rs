mod scope;
mod settings;
mod statement;
mod expression;

use arena::Arena;
use module::Module;
use ast::{StatementList, Ptr, Loc};

pub use self::scope::Scope;
pub use self::settings::Settings;

pub struct Transformer<'ast> {
    arena: &'ast Arena,
    settings: Settings,
    scope: Scope<'ast>,
}

pub trait Transformable<'ast> {
    fn transform(&self, t: &Transformer<'ast>);
}

impl<'ast> Transformer<'ast> {
    #[inline]
    pub fn transform<T: Transformable<'ast>>(&self, item: T) {
        item.transform(self);
    }

    #[inline]
    pub fn alloc<T: Copy + 'ast>(&self, item: T) -> Ptr<'ast, T> {
        Ptr::new(self.arena.alloc(item))
    }

    #[inline]
    pub fn swap<T: Copy + 'ast>(&self, ptr: &Ptr<'ast, Loc<T>>, item: T) {
        let new = self.arena.alloc(Loc {
            start: ptr.start,
            end: ptr.end,
            item
        });

        ptr.set(new);
    }
}

pub fn transform<'ast>(module: &mut Module, settings: Settings) {
    let arena = module.arena();
    let body = module.body();

    let transformer = Transformer {
        arena,
        settings,
        scope: Scope::new(arena),
    };

    transformer.transform(body);
}
