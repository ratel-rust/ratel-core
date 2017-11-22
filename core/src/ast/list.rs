use ast::Loc;
use ast::ptr::{Ptr, CopyCell};
use std::fmt::{self, Debug};
use arena::Arena;

/// Const-constructor that avoids issues with lifetimes.
/// Use List::empty() unless you need to create a 'static reference to a list.
#[macro_export]
macro_rules! empty_list {
    () => (List {
        root: $crate::ast::ptr::CopyCell {
            value: None
        }
    })
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ListItem<'ast, T: 'ast> {
    value: Ptr<'ast, T>,
    next: CopyCell<Option<&'ast ListItem<'ast, T>>>,
}

impl<'ast, T: Copy> Copy for ListItem<'ast, T> {}

pub struct ListBuilder<'ast, T: 'ast + Copy> {
    arena: &'ast Arena,
    first: &'ast ListItem<'ast, T>,
    last: &'ast ListItem<'ast, T>,
}

impl<'ast, T: 'ast + Copy> ListBuilder<'ast, T> {
    #[inline]
    pub fn new(arena: &'ast Arena, first: Ptr<'ast, T>) -> Self {
        let first = arena.alloc(ListItem {
            value: first,
            next: CopyCell::new(None)
        });

        ListBuilder {
            arena,
            first,
            last: first
        }
    }

    #[inline]
    pub fn push(&mut self, item: Ptr<'ast, T>) {
        let next = self.arena.alloc(ListItem {
            value: item,
            next: CopyCell::new(None)
        });

        self.last.next.set(Some(next));
        self.last = next;
    }

    #[inline]
    pub fn into_list(self) -> List<'ast, T> {
        List {
            root: CopyCell::new(Some(self.first))
        }
    }
}

pub struct EmptyListBuilder<'ast, T: 'ast + Copy> {
    arena: &'ast Arena,
    first: Option<&'ast ListItem<'ast, T>>,
    last: Option<&'ast ListItem<'ast, T>>,
}

impl<'ast, T: 'ast + Copy> EmptyListBuilder<'ast, T> {
    #[inline]
    pub fn new(arena: &'ast Arena) -> Self {
        EmptyListBuilder {
            arena,
            first: None,
            last: None,
        }
    }

    #[inline]
    pub fn push(&mut self, item: Ptr<'ast, T>) {
        match self.last {
            None => {
                self.first = Some(self.arena.alloc(ListItem {
                    value: item,
                    next: CopyCell::new(None)
                }));
                self.last = self.first;
            },
            Some(ref mut last) => {
                let next = self.arena.alloc(ListItem {
                    value: item,
                    next: CopyCell::new(None)
                });

                last.next.set(Some(next));
                *last = next;
            }
        }
    }

    #[inline]
    pub fn into_list(self) -> List<'ast, T> {
        List {
            root: CopyCell::new(self.first)
        }
    }
}

#[derive(Clone)]
pub struct List<'ast, T: 'ast> {
    pub(crate) root: CopyCell<Option<&'ast ListItem<'ast, T>>>,
}

impl<'ast, T: Copy> Copy for List<'ast, T> { }

#[derive(Debug, Clone, Copy)]
pub struct RawList {
    root: usize
}

impl RawList {
    pub unsafe fn into_list<'ast, T: 'ast>(self) -> List<'ast, T> {
        List {
            root: CopyCell::new(match self.root {
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
            root: CopyCell::new(None)
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
    pub fn ptr_iter(&self) -> ListPtrIter<'ast, T> {
        ListPtrIter {
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
    pub fn only_element(&self) -> Option<&'ast Loc<T>> {
        match self.root.get() {
            Some(&ListItem { ref value, ref next, .. }) => {
                match next.get() {
                    None => Some(&**value),
                    _    => None,
                }
            },
            None => None
        }
    }
}

impl<'ast, T: 'ast + Copy> List<'ast, T> {
    #[inline]
    pub fn from(arena: &'ast Arena, value: Ptr<'ast, T>) -> List<'ast, T> {
        List {
            root: CopyCell::new(Some(arena.alloc(ListItem {
                value,
                next: CopyCell::new(None)
            })))
        }
    }

    pub fn from_iter<I>(arena: &'ast Arena, source: I) -> List<'ast, T> where
        I: IntoIterator<Item = Loc<T>>
    {
        let mut iter = source.into_iter();

        let mut builder = match iter.next() {
            Some(item) => ListBuilder::new(arena, Ptr::new(arena.alloc(item))),
            None       => return List::empty(),
        };

        for item in iter {
            builder.push(Ptr::new(arena.alloc(item)));
        }

        builder.into_list()
    }
}


impl<'ast, T: 'ast> IntoIterator for List<'ast, T> {
    type Item = &'ast Loc<T>;
    type IntoIter = ListIter<'ast, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'ast, T: 'ast> IntoIterator for &'a List<'ast, T> {
    type Item = &'ast Loc<T>;
    type IntoIter = ListIter<'ast, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'ast, T: 'ast> From<List<'ast, T>> for ListPtrIter<'ast, T> {
    #[inline]
    fn from(list: List<'ast, T>) -> Self {
        list.ptr_iter()
    }
}

pub struct ListIter<'ast, T: 'ast> {
    next: Option<&'ast ListItem<'ast, T>>
}

impl<'ast, T: 'ast> Iterator for ListIter<'ast, T> {
    type Item = &'ast Loc<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        next.map(|list_item| {
            let value = &*list_item.value;
            self.next = list_item.next.get();
            value
        })
    }
}

pub struct ListPtrIter<'ast, T: 'ast> {
    next: Option<&'ast ListItem<'ast, T>>
}

impl<'ast, T: 'ast> Iterator for ListPtrIter<'ast, T> {
    type Item = &'ast Ptr<'ast, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        next.map(|list_item| {
            let value = &list_item.value;
            self.next = list_item.next.get();
            value
        })
    }
}
