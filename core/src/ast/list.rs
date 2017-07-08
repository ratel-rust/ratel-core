use std::cell::Cell;
use std::fmt::{self, Debug};
use arena::Arena;

#[derive(Debug, Clone, PartialEq)]
struct ListItem<'ast, T: 'ast> {
    value: T,
    next: Cell<Option<&'ast ListItem<'ast, T>>>,
}

pub struct ListBuilder<'ast, T: 'ast> {
    arena: &'ast Arena,
    first: &'ast ListItem<'ast, T>,
    last: &'ast ListItem<'ast, T>,
}

impl<'ast, T: 'ast> ListBuilder<'ast, T> {
    #[inline]
    pub fn new(arena: &'ast Arena, first: T) -> Self {
        let first = arena.alloc(ListItem {
            value: first,
            next: Cell::new(None)
        });

        ListBuilder {
            arena,
            first,
            last: first
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        let next = self.arena.alloc(ListItem {
            value: item,
            next: Cell::new(None)
        });

        self.last.next.set(Some(next));
        self.last = next;
    }

    #[inline]
    pub fn into_list(self) -> List<'ast, T> {
        List {
            root: Cell::new(Some(self.first))
        }
    }
}

#[derive(Clone)]
pub struct List<'ast, T: 'ast> {
    root: Cell<Option<&'ast ListItem<'ast, T>>>,
}

#[derive(Clone, Copy)]
pub struct RawList {
    root: usize
}

impl RawList {
    pub unsafe fn into_list<'ast, T: 'ast>(self) -> List<'ast, T> {
        List {
            root: Cell::new(match self.root {
                0   => None,
                ptr => Some(&*(ptr as *const ListItem<'ast, T>))
            })
        }
    }
}

impl<'ast, T: 'ast + PartialEq> PartialEq for List<'ast, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'ast, T: 'ast + Debug> Debug for List<'ast, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'ast, T: 'ast> List<'ast, T> {
    #[inline]
    pub fn empty() -> Self {
        List {
            root: Cell::new(None)
        }
    }

    #[inline]
    pub fn clear(&self) {
        self.root.set(None);
    }

    #[inline]
    pub fn into_raw(self) -> RawList {
        RawList {
            root: match self.root.get() {
                Some(ptr) => ptr as *const ListItem<'ast, T> as usize,
                None      => 0
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> ListIter<'ast, T> {
        ListIter {
            next: self.root.get()
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.get().is_none()
    }

    /// Returns the first element if, and only if, the list contains
    /// just that single element.
    #[inline]
    pub fn only_element(&self) -> Option<&'ast T> {
        match self.root.get() {
            Some(&ListItem { ref value, ref next, .. }) => {
                match next.get() {
                    None => Some(value),
                    _    => None,
                }
            },
            None => None
        }
    }

    #[inline]
    pub fn from(arena: &'ast Arena, value: T) -> List<'ast, T> {
        List {
            root: Cell::new(Some(arena.alloc(ListItem {
                value,
                next: Cell::new(None)
            })))
        }
    }

    pub fn from_iter<I>(arena: &'ast Arena, source: I) -> List<'ast, T> where
        I: IntoIterator<Item = T>
    {
        let mut iter = source.into_iter();

        let mut builder = match iter.next() {
            Some(item) => ListBuilder::new(arena, item),
            None       => return List::empty(),
        };

        for item in iter {
            builder.push(item);
        }

        builder.into_list()
    }
}


impl<'ast, T: 'ast> IntoIterator for List<'ast, T> {
    type Item = &'ast T;
    type IntoIter = ListIter<'ast, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'ast, T: 'ast> IntoIterator for &'a List<'ast, T> {
    type Item = &'ast T;
    type IntoIter = ListIter<'ast, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct ListIter<'ast, T: 'ast> {
    next: Option<&'ast ListItem<'ast, T>>
}

impl<'ast, T: 'ast> Iterator for ListIter<'ast, T> {
    type Item = &'ast T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        next.map(|list_item| {
            let ref value = list_item.value;
            self.next = list_item.next.get();
            value
        })
    }
}
