use std::fmt::{self, Debug};

use ratel::Module;
use ratel::ast::{Identifier, ExpressionNode};
use ratel_visitor::{Visitable, StaticVisitor, DynamicVisitor, ScopeKind};
use toolshed::{Arena, CopyCell};
use toolshed::list::GrowableList;
use toolshed::map::BloomMap;

/// Traverse the AST and produce a tree of `Scope`s.
pub fn analyze<'ast>(module: &'ast Module<'ast>) -> &'ast Scope<'ast> {
    let mut ctx = ScopeContext::new(module.arena());

    module.traverse(&ScopeAnalizer, &mut ctx);

    ctx.current.get()
}

pub type ReferenceData = ();

#[derive(Clone, Copy)]
pub struct Scope<'ast> {
    /// Kind of the scope.
    pub kind: ScopeKind,

    /// Whether or not the `super` keyword was used
    pub used_super: CopyCell<bool>,

    /// Whether or not the `this` keyword was used
    pub used_this: CopyCell<bool>,

    /// All references used in this scope
    pub used_refs: BloomMap<'ast, &'ast str, ReferenceData>,

    /// All references declared in this scope
    pub declared_refs: BloomMap<'ast, &'ast str, ReferenceData>,

    /// Parent scope of this scope
    pub parent: Option<&'ast Scope<'ast>>,

    /// All children of the this scope
    pub children: GrowableList<'ast, &'ast Scope<'ast>>,
}

impl<'ast> Scope<'ast> {
    pub fn new(kind: ScopeKind, parent: Option<&'ast Scope<'ast>>) -> Self {
        Scope {
            kind,
            used_super: CopyCell::new(false),
            used_this: CopyCell::new(false),
            used_refs: BloomMap::new(),
            declared_refs: BloomMap::new(),
            parent,
            children: GrowableList::new(),
        }
    }

    pub fn as_usize(&'ast self) -> usize {
        self as *const Scope as usize
    }

    pub unsafe fn from_usize(ptr: usize) -> &'ast Self {
        &*(ptr as *const Scope)
    }
}

/// Need to manually implement Debug to avoid circular reference on `parent`
impl<'ast> Debug for Scope<'ast> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Scope")
            .field("kind", &self.kind)
            .field("used_super", &self.used_super)
            .field("used_this", &self.used_this)
            .field("used_refs", &self.used_refs)
            .field("declared_refs", &self.declared_refs)
            .field("children", &self.children)
            .finish()
    }
}

/// Enough to check if the pointers are the same
impl<'ast> PartialEq for &'ast Scope<'ast> {
    fn eq(&self, other: &&'ast Scope<'ast>) -> bool {
        self.as_usize() == other.as_usize()
    }
}

struct ScopeContext<'ast> {
    arena: &'ast Arena,
    pub current: CopyCell<&'ast Scope<'ast>>,
}

impl<'ast> ScopeContext<'ast> {
    fn new(arena: &'ast Arena) -> Self {
        let current = CopyCell::new(
            arena.alloc(Scope::new(ScopeKind::Function, None))
        );

        ScopeContext {
            arena,
            current,
        }
    }
}

struct ScopeAnalizer;

impl<'ast> StaticVisitor<'ast> for ScopeAnalizer {
    type Context = ScopeContext<'ast>;

    fn on_enter_scope(kind: ScopeKind, ctx: &mut Self::Context) {
        ctx.current.set(
            ctx.arena.alloc(Scope::new(kind, Some(ctx.current.get())))
        );
    }

    fn on_leave_scope(ctx: &mut Self::Context) {
        let popped = ctx.current.get();

        ctx.current.set(popped.parent.unwrap());
        ctx.current.get().children.push(ctx.arena, popped);
    }

    fn on_reference_use(ident: &Identifier<'ast>, ctx: &mut Self::Context) {
        ctx.current.get().used_refs.insert(ctx.arena, *ident, ());
    }

    fn on_reference_declaration(ident: &Identifier<'ast>, ctx: &mut Self::Context) {
        ctx.current.get().declared_refs.insert(ctx.arena, *ident, ());
    }

    fn on_this_expression(_: &ExpressionNode<'ast>, ctx: &mut Self::Context) {
        ctx.current.get().used_this.set(true);
    }

    fn register(dv: &mut DynamicVisitor<'ast, Self::Context>) {
        dv.on_enter_scope.push(Self::on_enter_scope);
        dv.on_leave_scope.push(Self::on_leave_scope);
        dv.on_reference_use.push(Self::on_reference_use);
        dv.on_reference_declaration.push(Self::on_reference_declaration);
        dv.on_this_expression.push(Self::on_this_expression);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ratel::parse;

    #[test]
    fn scope_analysis() {
        let module = parse("function foo(bar) { doge; { moon; } return 10; }").unwrap();
        let root = analyze(&module);

        assert_eq!(root.parent, None);
        assert_eq!(root.kind, ScopeKind::Function);
        assert_eq!(root.used_refs.is_empty(), true);
        assert_eq!(root.declared_refs.contains_key("foo"), true);

        let foo = *root.children.as_list().only_element().unwrap();

        assert_eq!(foo.parent, Some(root));
        assert_eq!(foo.kind, ScopeKind::Function);
        assert_eq!(foo.used_refs.contains_key("doge"), true);
        assert_eq!(foo.declared_refs.contains_key("bar"), true);

        let moon = *foo.children.as_list().only_element().unwrap();

        assert_eq!(moon.parent, Some(foo));
        assert_eq!(moon.kind, ScopeKind::Block);
        assert_eq!(moon.used_refs.contains_key("moon"), true);
        assert_eq!(moon.declared_refs.is_empty(), true);
        assert_eq!(moon.children.as_list().is_empty(), true);
    }
}
