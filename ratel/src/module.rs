use toolshed::list::UnsafeList;
use toolshed::Arena;
use ast::StatementList;
use std::marker::PhantomData;

/// A JavaScript module parsed to an AST.
pub struct Module<'ast> {
    body: UnsafeList,
    arena: Arena,
    _phantom: PhantomData<&'ast StatementList<'ast>>
}

impl<'ast> Module<'ast> {
    #[inline]
    pub(crate) fn new(body: UnsafeList, arena: Arena) -> Self {
        Module {
            body,
            arena,
            _phantom: PhantomData,
        }
    }

    /// Get the body of the module as a list of statements.
    #[inline]
    pub fn body(&self) -> StatementList<'ast> {
        unsafe { self.body.into_list() }
    }

    /// Get a reference to the `Arena` on which the AST is allocated.
    #[inline]
    pub fn arena(&'ast self) -> &'ast Arena {
        &self.arena
    }
}
