use std::cell::Cell;
use std::ops::Deref;
use std::fmt::{self, Debug};

/// `Ptr` is a specialized `Cell` that holds a reference to T instead of T.
/// `Ptr` has defined lifetime and implements `Defer<Target = T>` for convenience.
#[derive(Clone)]
pub struct Ptr<'ast, T: 'ast> {
    inner: Cell<&'ast T>
}

impl<'ast, T: 'ast> Ptr<'ast, T> {
    #[inline]
    pub fn new(val: &'ast T) -> Self {
        Ptr {
            inner: Cell::new(val)
        }
    }
}

impl<'ast, T: 'ast> Deref for Ptr<'ast, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.inner.get()
    }
}

impl<'ast, T: 'ast + PartialEq> PartialEq for Ptr<'ast, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl<'ast, T: 'ast + Debug> Debug for Ptr<'ast, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}
