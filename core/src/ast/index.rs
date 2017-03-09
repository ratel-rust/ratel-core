/// Just an alias
pub type Index = usize;

/// Max usize value is treated as null inside `OptIndex`
const NULL: Index = !0;

/// A fast replacement for `Option<Index>`.
///
/// Due to alignment `Option<Index>` will almost always consume two
/// words in memory, which is both memory bloat and perf hit.
///
/// Since the actual internal `Index` is not visible, this is just
/// as safe as regular `Option`, consumer has to use either the
/// `map` or `as_option` method to get to the actual value of it.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OptIndex(Index);

impl OptIndex {
    #[inline]
    pub fn new(index: Index) -> Self {
        OptIndex(index)
    }

    #[inline]
    pub fn map<T, F>(&self, f: F) -> Option<T>
        where F: FnOnce(usize) -> T
    {
        if !self.is_null() {
            Some(f(self.0))
        } else {
            None
        }
    }

    #[inline]
    pub fn as_option(&self) -> Option<Index> {
        if !self.is_null() {
            Some(self.0)
        } else {
            None
        }
    }

    #[inline]
    pub fn null() -> Self {
        OptIndex(NULL)
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.0 == NULL
    }
}

#[inline]
pub fn null() -> OptIndex {
    OptIndex(NULL)
}

#[inline]
pub fn idx(index: Index) -> OptIndex {
    OptIndex(index)
}

impl From<Index> for OptIndex {
    #[inline(always)]
    fn from(val: Index) -> Self {
        OptIndex(val)
    }
}
