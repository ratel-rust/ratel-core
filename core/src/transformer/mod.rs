mod scope;
mod settings;
mod statement;
mod expression;

use arena::Arena;
use module::Module;
use ast::{Ptr, Loc, StatementPtr, ExpressionPtr};

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

pub trait Visitor {
    #[inline]
    fn visit_statement<'ast>(&self, StatementPtr<'ast>, &Transformer<'ast>) {}

    #[inline]
    fn visit_expression<'ast>(&self, ExpressionPtr<'ast>, &Transformer<'ast>) {}
}

impl<A: Visitor, B: Visitor> Visitor for (A, B) {
    #[inline]
    fn visit_statement<'ast>(&self, statement: StatementPtr<'ast>, t: &Transformer<'ast>) {
        A::visit_statement(&self.0, statement, t);
        B::visit_statement(&self.1, statement, t);
    }

    #[inline]
    fn visit_expression<'ast>(&self, expression: ExpressionPtr<'ast>, t: &Transformer<'ast>) {
        A::visit_expression(&self.0, expression, t);
        B::visit_expression(&self.1, expression, t);
    }
}

impl Visitor for Vec<Box<Visitor>> {
    #[inline]
    fn visit_statement<'ast>(&self, statement: StatementPtr<'ast>, t: &Transformer<'ast>) {
        for visitor in self {
            visitor.visit_statement(statement, t);
        }
    }

    #[inline]
    fn visit_expression<'ast>(&self, expression: ExpressionPtr<'ast>, t: &Transformer<'ast>) {
        for visitor in self {
            visitor.visit_expression(expression, t);
        }
    }
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
