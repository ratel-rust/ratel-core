use toolshed::CopyCell;
use std::ops::Deref;
use std::fmt::{self, Debug};
use ast::Loc;

/// `Node` is a specialized `Cell` that holds a reference to T instead of T.
/// `Node` has defined lifetime and implements `Defer<Target = T>` for convenience.
#[derive(Clone, Copy)]
pub struct Node<'ast, T: 'ast> {
    inner: CopyCell<&'ast Loc<T>>
}

impl<'ast, T: 'ast> Node<'ast, T> {
    #[inline]
    pub fn new(ptr: &'ast Loc<T>) -> Self {
        Node {
            inner: CopyCell::new(ptr)
        }
    }

    #[inline]
    pub fn set(&self, ptr: &'ast Loc<T>) {
        self.inner.set(ptr)
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut &'ast Loc<T> {
        self.inner.get_mut()
    }
}

impl<'ast, T: 'ast> Deref for Node<'ast, T> {
    type Target = Loc<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.get()
    }
}

impl<'ast, T: 'ast + PartialEq> PartialEq for Node<'ast, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl<'ast, T: 'ast + Debug> Debug for Node<'ast, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ptr() {
        let one = Loc::new(0, 0, "one");
        let two = Loc::new(0, 0, "two");

        let one_ptr = Node::new(&one);
        let two_ptr = one_ptr;

        assert_eq!(*one_ptr, Loc::new(0, 0, "one"));
        assert_eq!(*two_ptr, Loc::new(0, 0, "one"));

        two_ptr.set(&two);

        assert_eq!(*one_ptr, Loc::new(0, 0, "one"));
        assert_eq!(*two_ptr, Loc::new(0, 0, "two"));
    }
}
