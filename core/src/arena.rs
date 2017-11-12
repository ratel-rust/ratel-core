use std::mem::size_of;
use std::cell::Cell;

const ARENA_BLOCK: usize = 64 * 1024;

pub struct Arena {
    store: Cell<Vec<Vec<u8>>>,
    ptr: Cell<*mut u8>,
    offset: Cell<usize>
}

impl Arena {
    pub fn new() -> Self {
        let mut store = vec![Vec::with_capacity(ARENA_BLOCK)];
        let ptr = store[0].as_mut_ptr();

        Arena {
            store: Cell::new(store),
            ptr: Cell::new(ptr),
            offset: Cell::new(0)
        }
    }

    #[inline]
    pub fn alloc<'a, T: Sized + Copy>(&'a self, val: T) -> &'a T {
        let mut offset = self.offset.get();
        let cap = offset + size_of::<T>();

        if cap > ARENA_BLOCK {
            self.grow();

            offset = 0;
            self.offset.set(size_of::<T>());
        } else {
            self.offset.set(cap);
        }

        unsafe {
            let ptr = self.ptr.get().offset(offset as isize) as *mut T;
            *ptr = val;
            &*ptr
        }
    }

    pub fn alloc_str<'a>(&'a self, val: &str) -> &'a str {
        let offset = self.offset.get();
        let alignment = size_of::<usize>() - (val.len() % size_of::<usize>());
        let cap = offset + val.len() + alignment;

        if cap > ARENA_BLOCK {
            return self.alloc_string(val.into());
        }

        self.offset.set(cap);

        unsafe {
            use std::ptr::copy_nonoverlapping;
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            let ptr = self.ptr.get().offset(offset as isize);
            copy_nonoverlapping(val.as_ptr(), ptr, val.len());

            from_utf8_unchecked(from_raw_parts(ptr, val.len()))
        }
    }

    pub fn alloc_str_zero_end<'a>(&'a self, val: &str) -> *const u8 {
        let len_with_zero = val.len() + 1;
        let offset = self.offset.get();
        let alignment = size_of::<usize>() - (len_with_zero % size_of::<usize>());
        let cap = offset + len_with_zero + alignment;

        if cap > ARENA_BLOCK {
            let mut vec = Vec::with_capacity(len_with_zero);
            vec.extend_from_slice(val.as_bytes());
            vec.push(0);
            return self.alloc_bytes(vec);
        }

        self.offset.set(cap);

        unsafe {
            use std::ptr::copy_nonoverlapping;

            let ptr = self.ptr.get().offset(offset as isize);
            copy_nonoverlapping(val.as_ptr(), ptr, val.len());
            *ptr.offset(val.len() as isize) = 0;
            ptr
        }
    }

    pub fn alloc_string<'a>(&'a self, val: String) -> &'a str {
        let len = val.len();
        let ptr = self.alloc_bytes(val.into_bytes());

        unsafe {
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            from_utf8_unchecked(from_raw_parts(ptr, len))
        }
    }

    #[inline]
    pub fn alloc_bytes(&self, val: Vec<u8>) -> *const u8 {
        let ptr = val.as_ptr();

        let mut temp = self.store.replace(Vec::new());
        temp.push(val);
        self.store.replace(temp);

        ptr
    }

    fn grow(&self) {
        let mut temp = self.store.replace(Vec::new());
        let mut block = Vec::with_capacity(ARENA_BLOCK);
        self.ptr.set(block.as_mut_ptr());
        temp.push(block);
        self.store.replace(temp);
    }
}
