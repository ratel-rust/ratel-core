use toolshed::list::UnsafeList;
use toolshed::Arena;
use ast::StatementList;

pub struct Module {
    body: UnsafeList,
    arena: Arena,
}

impl Module {
    #[inline]
    pub fn new(body: UnsafeList, arena: Arena) -> Self {
        Module {
            body,
            arena,
        }
    }

    #[inline]
    pub fn body<'ast>(&'ast self) -> StatementList<'ast> {
        unsafe { self.body.into_list() }
    }

    #[inline]
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
}
