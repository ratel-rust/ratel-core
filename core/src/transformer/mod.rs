mod scope;
mod settings;
mod statement;
mod expression;

use arena::Arena;
use module::Module;
use ast::{List, Ptr, Loc};

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
}

impl<'ast, T: Transformable<'ast>> Transformable<'ast> for List<'ast, T> {
    #[inline]
    fn transform(&self, t: &Transformer<'ast>) {
        for item in self.iter() {
            item.transform(t);
        }
    }
}

impl<'ast, T: Transformable<'ast>> Transformable<'ast> for Ptr<'ast, T> {
    #[inline]
    fn transform(&self, t: &Transformer<'ast>) {
        (**self).transform(t);
    }
}

impl<'ast, T: Transformable<'ast>> Transformable<'ast> for Loc<T> {
    #[inline]
    fn transform(&self, t: &Transformer<'ast>) {
        self.item.transform(t);
    }
}

pub fn transform<'ast>(module: Module, settings: Settings) {
    let arena = module.arena();
    let body = module.body();

    let transformer = Transformer {
        arena,
        settings,
        scope: Scope::new(arena),
    };

    transformer.transform(body);
}
