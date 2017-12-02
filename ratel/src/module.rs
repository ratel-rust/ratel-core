use toolshed::list::UnsafeList;
use toolshed::Arena;
use ast::StatementList;
use std::marker::PhantomData;

pub struct Module<'ast> {
    body: UnsafeList,
    arena: Arena,
    _phantom: PhantomData<&'ast StatementList<'ast>>
}

impl<'ast> Module<'ast> {
    #[inline]
    pub fn new(body: UnsafeList, arena: Arena) -> Self {
        Module {
            body,
            arena,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn body(&self) -> StatementList<'ast> {
        unsafe { self.body.into_list() }
    }

    #[inline]
    pub fn arena(&'ast self) -> &'ast Arena {
        &self.arena
    }
}
