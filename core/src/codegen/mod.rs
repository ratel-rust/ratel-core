use ast::{Index, OptIndex, Item, Program};

/// The `Generator` is a wrapper around an owned `String` that's used to
/// stringify the AST. There is a bunch of useful methods here to manage
/// things like indentation and automatically producing minified code.
struct Generator<'src> {
    program: &'src Program<'src>,
    pub minify: bool,
    code: Vec<u8>,
    dent: u16,
}

impl<'src> Generator<'src> {
    pub fn new(program: &'src Program<'src>, minify: bool) -> Self {
        Generator {
            program: program,
            minify: minify,
            code: Vec::with_capacity(128),
            dent: 0,
        }
    }

    #[inline]
    pub fn new_line(&mut self) {
        if !self.minify {
            self.write_byte(b'\n');
            for _ in 0..self.dent {
                self.write_bytes(b"    ");
            }
        }
    }

    #[inline]
    pub fn write<T: ToCode>(&mut self, item: &T) {
        item.to_code(self);
    }

    #[inline]
    pub fn write_byte(&mut self, ch: u8) {
        self.code.push(ch);
    }

    #[inline]
    pub fn write_bytes(&mut self, slice: &[u8]) {
        extend_from_slice(&mut self.code, slice);
    }

    #[inline]
    pub fn write_min(&mut self, slice: &[u8], minslice: &[u8]) {
        if self.minify {
            self.write_bytes(minslice);
        } else {
            self.write_bytes(slice);
        }
    }

    #[inline]
    pub fn write_list<T: ToCode>(&mut self, items: &Vec<T>) {
        let mut iter = items.iter();

        for item in iter.next() {
            self.write(item);
        }

        for item in iter {
            self.write_min(b", ", b",");
            self.write(item);
        }
    }

    #[inline]
    pub fn write_block<T: ToCode>(&mut self, items: &Vec<T>) {
        if items.len() == 0 {
            return;
        }

        self.indent();
        for item in items {
            self.new_line();
            self.write(item);
        }
        self.dedent();
        self.new_line();
    }

    #[inline]
    fn write_from_index(&mut self, index: Index) {
        self.program[index].to_code(self);
    }

    #[inline]
    fn write_from_optindex(&mut self, opt: OptIndex) {
        opt.map(|index| self.write_from_index(index));
    }

    // pub fn write_declaration_or_expression(&mut self, statement: &Statement) {
    //     match *statement {
    //         Statement::VariableDeclaration {
    //             ref kind,
    //             ref declarators,
    //         } => {
    //             self.write(kind);
    //             self.write_byte(b' ');
    //             self.write_list(declarators);
    //         },

    //         Statement::Expression {
    //             ref value,
    //         } => {
    //             value.to_code(self);
    //         },

    //         _ => panic!("Invalid AST structure!"),
    //     }
    // }

    #[inline]
    pub fn indent(&mut self) {
        self.dent += 1;
    }

    #[inline]
    pub fn dedent(&mut self) {
        self.dent -= 1;
    }

    #[inline]
    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

/// The `ToCode` trait provides an interface to pieces of grammar, that allows
/// to efficiently write characters and string slices to the code `Generator`.
trait ToCode {
    fn to_code(&self, gen: &mut Generator);
}

impl<'src> ToCode for Item<'src> {
    fn to_code(&self, gen: &mut Generator) {}
}

// From: https://github.com/dtolnay/fastwrite/blob/master/src/lib.rs#L68
//
// LLVM is not able to lower `Vec::extend_from_slice` into a memcpy, so this
// helps eke out that last bit of performance.
#[inline]
fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ::std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dst.as_mut_ptr().offset(dst_len as isize),
            src_len);
    }
}
