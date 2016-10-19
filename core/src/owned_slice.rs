use std::{ str, slice, fmt };
use std::ops::Deref;

/// This struct _should_ be unnecessary as it does exactly the same thing
/// a regular `&str` does. It does however remove the burden of handling
/// lifetimes within the code at the cost of making sure that the slice
/// doesn't outlive the source. Ideally the AST and all it's components
/// should contain a lifetime bound to the source string, however making
/// that work with current iteration of borrow checker is anywhere between
/// very hard to impossible.
///
/// Performance wise all casting should be compiled away since this struct
/// matches the format of &str exactly.
#[derive(Clone, Copy)]
pub struct OwnedSlice {
    ptr: *const u8,
    len: usize,
}

impl OwnedSlice {
    /// Create an `OwnedSlice` from any `&str`. This method is explicitly
    /// marked as unsafe - given Rust can't guarantee that the raw pointer
    /// stored internally doesn't turn into a dangling pointer at any time
    /// in the future.
    #[inline]
    pub unsafe fn from_str(source: &str) -> Self {
        OwnedSlice {
            ptr: source.as_ptr(),
            len: source.len(),
        }
    }

    /// Create an `OwnedSlice` from a `&'static str`. Since static slices
    /// are guaranteed to exist though the whole life of the program, there
    /// no risk that this slice will ever include a dangling pointer.
    #[inline]
    pub fn from_static(source: &'static str) -> Self {
        OwnedSlice {
            ptr: source.as_ptr(),
            len: source.len(),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.as_bytes())
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl From<&'static str> for OwnedSlice {
    #[inline]
    fn from(source: &'static str) -> Self {
        OwnedSlice {
            ptr: source.as_ptr(),
            len: source.len(),
        }
    }
}

impl Deref for OwnedSlice {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for OwnedSlice {
    #[inline]
    fn eq(&self, other: &OwnedSlice) -> bool {
        self.as_str() == other.as_str()
    }
}

impl fmt::Debug for OwnedSlice {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for OwnedSlice {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}
