use std::fmt::{self, Debug};

use ratel::Module;
use ratel::ast::{Identifier, ExpressionNode};
use ratel_visitor::{Visitable, ScopeKind, Visitor};
use toolshed::{Arena, CopyCell};
use toolshed::list::GrowableList;
use toolshed::map::BloomMap;

/// Traverse the AST and produce a tree of `Scope`s.
#[inline]
pub fn analyze<'ast>(module: &'ast Module<'ast>) -> &'ast Scope<'ast> {
    let mut visitor = ScopeAnalyzer::new(module.arena());

    module.visit_with(&mut visitor);

    visitor.current.get()
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
    #[inline]
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

    #[inline]
    pub fn as_usize(&'ast self) -> usize {
        self as *const Scope as usize
    }

    #[inline]
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
    #[inline]
    fn eq(&self, other: &&'ast Scope<'ast>) -> bool {
        self.as_usize() == other.as_usize()
    }
}

struct ScopeAnalyzer<'ast> {
    arena: &'ast Arena,
    pub current: CopyCell<&'ast Scope<'ast>>,
}

impl<'ast> ScopeAnalyzer<'ast> {
    #[inline]
    fn new(arena: &'ast Arena) -> Self {
        let current = CopyCell::new(
            arena.alloc(Scope::new(ScopeKind::Function, None))
        );

        ScopeAnalyzer {
            arena,
            current,
        }
    }
}

impl<'ast> Visitor<'ast> for ScopeAnalyzer<'ast> {
    #[inline]
    fn on_enter_scope(&mut self, kind: ScopeKind) {
        self.current.set(
            self.arena.alloc(Scope::new(kind, Some(self.current.get())))
        );
    }

    #[inline]
    fn on_leave_scope(&mut self) {
        let popped = self.current.get();

        self.current.set(popped.parent.unwrap());
        self.current.get().children.push(self.arena, popped);
    }

    #[inline]
    fn on_reference_use(&mut self, ident: &Identifier<'ast>) {
        self.current.get().used_refs.insert(self.arena, *ident, ());
    }

    #[inline]
    fn on_reference_declaration(&mut self, ident: &Identifier<'ast>) {
        self.current.get().declared_refs.insert(self.arena, *ident, ());
    }

    #[inline]
    fn on_this_expression(&mut self, _: &ExpressionNode<'ast>) {
        self.current.get().used_this.set(true);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ratel::parse;

    #[test]
    fn scope_analysis() {
        let module = parse("function test_function(bar) { doge; { moon; } return 10; }").unwrap();
        let root = analyze(&module);

        assert_eq!(root.parent, None);
        assert_eq!(root.kind, ScopeKind::Function);
        assert_eq!(root.used_refs.is_empty(), true);
        assert_eq!(root.declared_refs.contains_key("test_function"), true);

        let test_function = *root.children.as_list().only_element().unwrap();

        assert_eq!(test_function.parent, Some(root));
        assert_eq!(test_function.kind, ScopeKind::Function);
        assert_eq!(test_function.used_refs.contains_key("doge"), true);
        assert_eq!(test_function.declared_refs.contains_key("bar"), true);

        let moon = *test_function.children.as_list().only_element().unwrap();

        assert_eq!(moon.parent, Some(test_function));
        assert_eq!(moon.kind, ScopeKind::Block);
        assert_eq!(moon.used_refs.contains_key("moon"), true);
        assert_eq!(moon.declared_refs.is_empty(), true);
        assert_eq!(moon.children.as_list().is_empty(), true);
    }
}
