use std::ops::Deref;
use std::fmt::{self, Debug};
use ast::Loc;

/// This should be identical to the `Cell` implementation in the standard
/// library, but always require that the internal type implements `Copy`
/// and implements Copy itself.
#[derive(Debug, PartialEq)]
pub struct CopyCell<T: Copy> {
    pub(crate) value: T
}

impl<T: Copy> CopyCell<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        CopyCell {
            value
        }
    }

    #[inline]
    fn mut_ptr(&self) -> *mut T {
        &self.value as *const T as *mut T
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe {
            *self.mut_ptr()
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.mut_ptr()
        }
    }

    #[inline]
    pub fn set(&self, value: T) {
        let ptr = unsafe { &mut *self.mut_ptr() };
        *ptr = value;
    }
}

impl<T: Copy> Clone for CopyCell<T> {
    #[inline]
    fn clone(&self) -> CopyCell<T> {
        CopyCell::new(self.get())
    }
}

impl<T: Copy> Copy for CopyCell<T> { }

/// `Ptr` is a specialized `Cell` that holds a reference to T instead of T.
/// `Ptr` has defined lifetime and implements `Defer<Target = T>` for convenience.
#[derive(Clone, Copy)]
pub struct Ptr<'ast, T: 'ast> {
    inner: CopyCell<&'ast Loc<T>>
}

impl<'ast, T: 'ast> Ptr<'ast, T> {
    #[inline]
    pub fn new(ptr: &'ast Loc<T>) -> Self {
        Ptr {
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

impl<'ast, T: 'ast> Deref for Ptr<'ast, T> {
    type Target = Loc<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ptr() {
        let foo = Loc::new(0, 0, "foo");
        let bar = Loc::new(0, 0, "bar");

        let foo_ptr = Ptr::new(&foo);
        let bar_ptr = foo_ptr.clone();

        assert_eq!(*foo_ptr, Loc::new(0, 0, "foo"));
        assert_eq!(*bar_ptr, Loc::new(0, 0, "foo"));

        bar_ptr.set(&bar);

        assert_eq!(*foo_ptr, Loc::new(0, 0, "foo"));
        assert_eq!(*bar_ptr, Loc::new(0, 0, "bar"));
    }
}
