use toolshed::list::RawList;
use toolshed::Arena;
use ast::StatementList;

pub struct Module {
    body: RawList,
    arena: Arena,
}

impl Module {
    #[inline]
    pub fn new(body: RawList, arena: Arena) -> Self {
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
