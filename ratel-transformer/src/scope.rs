use ratel::ast::Identifier;
use ratel_visitor::{StaticVisitor, DynamicVisitor, ScopeKind};
use toolshed::{Arena, CopyCell};
use toolshed::list::{List, GrowableList};
use toolshed::map::BloomMap;

pub type ReferenceData = ();

#[derive(Clone, Copy, Debug)]
pub struct Scope<'ast> {
    /// Kind of the scope.
    kind: ScopeKind,

    /// Whether or not the `super` keyword was used
    used_super: CopyCell<bool>,

    /// Whether or not the `this` keyword was used
    used_this: CopyCell<bool>,

    /// All references used in this scope
    used_refs: BloomMap<'ast, &'ast str, ReferenceData>,

    /// All references declared in this scope
    declared_refs: BloomMap<'ast, &'ast str, ReferenceData>,

    children: GrowableList<'ast, &'ast Scope<'ast>>,
}

impl<'ast> Scope<'ast> {
    #[inline]
    fn new(kind: ScopeKind) -> Self {
        Scope {
            kind,
            used_super: CopyCell::new(false),
            used_this: CopyCell::new(false),
            used_refs: BloomMap::new(),
            declared_refs: BloomMap::new(),
            children: GrowableList::new(),
        }
    }
}

pub struct ScopeContext<'ast> {
    arena: &'ast Arena,
    current: CopyCell<&'ast Scope<'ast>>,
    pub stack: List<'ast, Scope<'ast>>,
}

impl<'ast> ScopeContext<'ast> {
    pub fn new(arena: &'ast Arena) -> Self {
        let stack = List::empty();
        let current = CopyCell::new(stack.prepend(arena, Scope::new(ScopeKind::Function)));

        ScopeContext {
            arena,
            current,
            stack,
        }
    }
}

pub struct ScopeAnalizer;

impl<'ast> StaticVisitor<'ast> for ScopeAnalizer {
    type Context = ScopeContext<'ast>;

    #[inline]
    fn on_enter_scope(kind: ScopeKind, ctx: &mut Self::Context) {
        ctx.current.set(ctx.stack.prepend(ctx.arena, Scope::new(kind)));
    }

    #[inline]
    fn on_leave_scope(ctx: &mut Self::Context) {
        let popped = ctx.stack.shift().unwrap();

        ctx.current.set(ctx.stack.first_element().unwrap());
        ctx.current.get().children.push(ctx.arena, popped);
    }

    #[inline]
    fn on_reference_use(ident: &Identifier<'ast>, ctx: &mut Self::Context) {
        ctx.current.get().used_refs.insert(ctx.arena, *ident, ());
    }

    #[inline]
    fn on_reference_declaration(ident: &Identifier<'ast>, ctx: &mut Self::Context) {
        ctx.current.get().declared_refs.insert(ctx.arena, *ident, ());
    }

    #[inline]
    fn register(dv: &mut DynamicVisitor<'ast, Self::Context>) {
        dv.on_enter_scope.push(Self::on_enter_scope);
        dv.on_leave_scope.push(Self::on_leave_scope);
        dv.on_reference_use.push(Self::on_reference_use);
        dv.on_reference_declaration.push(Self::on_reference_declaration);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ratel::parse;
    use ratel_visitor::Visitable;

    #[test]
    fn scope_analysis() {
        let module = parse("function foo(bar) { doge; { moon; } return 10; }").unwrap();

        let mut ctx = ScopeContext::new(module.arena());

        module.traverse(&ScopeAnalizer, &mut ctx);

        let root = ctx.stack.shift().unwrap();

        assert_eq!(ctx.stack.is_empty(), true);

        assert_eq!(root.kind, ScopeKind::Function);
        assert_eq!(root.used_refs.is_empty(), true);
        assert_eq!(root.declared_refs.contains_key("foo"), true);

        let foo = root.children.as_list().only_element().unwrap();

        assert_eq!(foo.kind, ScopeKind::Function);
        assert_eq!(foo.used_refs.contains_key("doge"), true);
        assert_eq!(foo.declared_refs.contains_key("bar"), true);

        let moon = foo.children.as_list().only_element().unwrap();

        assert_eq!(moon.kind, ScopeKind::Block);
        assert_eq!(moon.used_refs.contains_key("moon"), true);
        assert_eq!(moon.declared_refs.is_empty(), true);
        assert_eq!(moon.children.as_list().is_empty(), true);
    }
}
