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
