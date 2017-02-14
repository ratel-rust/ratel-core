use ast::OperatorKind;
use std::{slice, str, ptr, mem};
use std::marker::PhantomData;

const INLINE_STR_CAP: usize = 22;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Slice(pub usize, pub usize);

impl Slice {
    #[inline(always)]
    pub fn as_str<'id, 'src>(&'id self, src: &'src str) -> &'src str {
        unsafe { src.slice_unchecked(self.0, self.1) }
        // &src[self.0..self.1]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.1 - self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct InlineStr<'src> {
    len: u8,
    buf: [u8; INLINE_STR_CAP],
    phantom: PhantomData<&'src u8>
}

impl<'src> InlineStr<'src> {
    /// Create a new empty InlineStr
    #[inline]
    pub fn new() -> Self {
        let mut s: InlineStr = unsafe { mem::uninitialized() };

        s.len = 0;
        s
    }

    /// Get the maximum size the `InlineStr` can hold, this is a fixed number.
    #[inline]
    pub fn cap() -> usize {
        INLINE_STR_CAP
    }

    /// Creates an InlineStr from a &str. This will drop any characters that don't fit on the
    /// internal buffer (InlineStr::cap(), default 22).
    pub fn from_str(src: &str) -> Self {
        let len = match src.len() <= INLINE_STR_CAP {
            true => src.len(),
            false => {
                let mut len = INLINE_STR_CAP;

                // Make sure we are slicing the &str at character boundary
                while !src.is_char_boundary(len - 1) && len > 0 {
                    len -= 1;
                }

                len
            }
        };

        let mut s: InlineStr;

        unsafe {
            s = mem::uninitialized();
            ptr::copy_nonoverlapping(src.as_ptr(), s.buf.as_mut_ptr(), len);
            s.len = len as u8;
        }

        s
    }

    /// Cheaply converts `InlineStr` to `&str`.
    #[inline]
    pub fn as_str(&self) -> &'src str {
        unsafe {
            str::from_utf8_unchecked(
                slice::from_raw_parts(self.buf.as_ptr(), self.len as usize)
            )
        }
    }

    /// Push a byte onto `InlineStr`. This will panic in debug mode and lead to undefined
    /// behavior should capacity of the buffer be exceeded.
    #[inline]
    pub unsafe fn push(&mut self, byte: u8) {
        debug_assert!(INLINE_STR_CAP > self.len as usize);

        let len = self.len;
        self.set(len as usize, byte);
        self.len += 1;
    }

    /// Create a new `InlineStr` from raw components. This can be useful for creating
    /// static strings
    #[inline]
    pub unsafe fn from_raw_parts(buf: [u8; INLINE_STR_CAP], len: u8) -> Self {
        debug_assert!(INLINE_STR_CAP >= len as usize);

        InlineStr {
            buf: buf,
            len: len,
            phantom: PhantomData,
        }
    }


    #[inline]
    pub unsafe fn set_len(&mut self, len: u8) {
        debug_assert!(len as usize <= INLINE_STR_CAP);

        self.len = len;
    }

    #[inline]
    pub unsafe fn get(&mut self, index: usize) -> u8 {
        debug_assert!(index < INLINE_STR_CAP);

        *self.buf.as_ptr().offset(index as isize)
    }

    #[inline]
    pub unsafe fn set(&mut self, index: usize, byte: u8) {
        debug_assert!(index < INLINE_STR_CAP);

        *self.buf.as_mut_ptr().offset(index as isize) = byte;
    }

    #[inline]
    pub unsafe fn shift_left(&mut self, count: usize) {
        debug_assert!(count <= INLINE_STR_CAP);

        ptr::copy(self.buf.as_ptr().offset(count as isize), self.buf.as_mut_ptr(), INLINE_STR_CAP - count);
    }
}

impl<'src> From<&'src str> for InlineStr<'src> {
    #[inline]
    fn from(val: &str) -> InlineStr {
        InlineStr::from_str(val)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ident<'src> {
    /// Reference of the identifier in the source code.
    Insitu(&'src str),

    /// Inline-allocated string. Useful for dynamically created identifiers (mangler).
    Inline(InlineStr<'src>),
}

impl<'src> Ident<'src> {
    #[inline]
    pub fn as_str(&self) -> &'src str {
        match *self {
            Ident::Insitu(s) => s,
            Ident::Inline(ref s) => s.as_str()
        }
    }

    #[inline]
    pub fn equals(&self, other: Ident) -> bool {
        self.as_str() == other.as_str()
    }

    /// Create a new identifier from an index number using the 26 basic ASCII alphabet
    /// letters in both lower and upper case, such as:
    ///
    /// ```toml
    /// 0  -> "a"
    /// 1  -> "b"
    /// 2  -> "c"
    /// ...
    /// 24 -> "y"
    /// 25 -> "z"
    /// 26 -> "A"
    /// 27 -> "B"
    /// 28 -> "C"
    /// ...
    /// 50 -> "Y"
    /// 51 -> "Z"
    /// 52 -> "aa"
    /// 53 -> "ba"
    /// 54 -> "ca"
    /// ```
    ///
    /// Note that unlike decimal numbers, the format is little-endian for better performance.
    pub fn unique(mut id: u64) -> Ident<'src> {
        static ALPHA: [u8; 52] = *b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let mut s = InlineStr::new();

        while id >= 52 {
            // At 22 bytes with 52 possibilities, we will exhaust u64 without ever getting
            // close to overloading the buffer.
            unsafe {
                s.push(ALPHA[(id % 52) as usize]);
            }
            id /= 52;

            if id == 0 {
                break;
            }
        }

        s.into()
    }

    /// Converts non-inline identifier to an inline one and returns a mutable reference to it.
    ///
    /// **Note:** While this doesn't panic and is generally safe, it will drop excess characters
    /// which don't fit on `InlineStr`.
    pub fn get_mut_inline_str(&mut self) -> &mut InlineStr<'src> {
        *self = match *self {
            Ident::Insitu(s)         => Ident::Inline(s.into()),
            Ident::Inline(ref mut s) => return s,
        };

        match *self {
            Ident::Inline(ref mut s) => s,
            _                        => unreachable!()
        }
    }
}

impl<'src> From<&'src str> for Ident<'src> {
    #[inline]
    fn from(s: &'src str) -> Self {
        Ident::Insitu(s)
    }
}

impl<'src> From<InlineStr<'src>> for Ident<'src> {
    #[inline]
    fn from(s: InlineStr<'src>) -> Self {
        Ident::Inline(s)
    }
}

// #[derive(Debug, PartialEq, Clone, Copy)]
// pub enum Value {
//     Undefined,
//     Null,
//     Boolean(bool),
//     Number(Slice),
//     Binary(u64),
//     String(Slice),
//     RawQuasi(Slice),
// }

// #[derive(Debug, PartialEq, Clone)]
// pub struct Parameter {
//     pub name: Ident,
//     pub default: Option<Box<Expression>>
// }

// #[derive(Debug, PartialEq, Clone)]
// pub enum Expression {
//     Void,
//     This,
//     Identifier(Ident),
//     Literal(Value),
//     Template {
//         tag: Option<Box<Expression>>,
//         expressions: Vec<Expression>,
//         quasis: Vec<Slice>,
//     },
//     RegEx {
//         pattern: Slice,
//         flags: Slice
//     },
//     Array(Vec<Expression>),
//     Sequence(Vec<Expression>),
//     Object(Vec<ObjectMember>),
//     Member {
//         object: Box<Expression>,
//         property: Ident,
//     },
//     ComputedMember {
//         object: Box<Expression>,
//         property: Box<Expression>,
//     },
//     Call {
//         callee: Box<Expression>,
//         arguments: Vec<Expression>,
//     },
//     Binary {
//         parenthesized: bool,
//         operator: OperatorKind,
//         left: Box<Expression>,
//         right: Box<Expression>,
//     },
//     Prefix {
//         operator: OperatorKind,
//         operand: Box<Expression>,
//     },
//     Postfix {
//         operator: OperatorKind,
//         operand: Box<Expression>,
//     },
//     Conditional {
//         test: Box<Expression>,
//         consequent: Box<Expression>,
//         alternate: Box<Expression>,
//     },
//     ArrowFunction {
//         params: Vec<Parameter>,
//         body: Box<Statement>,
//     },
//     Function {
//         name: Option<Ident>,
//         params: Vec<Parameter>,
//         body: Vec<Statement>,
//     },
//     Class {
//         name: Option<Ident>,
//         extends: Option<Ident>,
//         body: Vec<ClassMember>,
//     },
// }

// impl Expression {
//     pub fn binding_power(&self) -> u8 {
//         match *self {
//             Expression::Member {
//                 ..
//             }
//             |
//             Expression::ArrowFunction {
//                 ..
//             } => 18,

//             Expression::Call {
//                 ..
//             } => 17,

//             Expression::Prefix {
//                 ..
//             } => 15,

//             Expression::Binary {
//                 ref operator,
//                 ..
//             }
//             |
//             Expression::Postfix {
//                 ref operator,
//                 ..
//             } => operator.binding_power(),

//             Expression::Conditional {
//                 ..
//             } => 4,

//             _  => 100,
//         }
//     }

//     #[inline]
//     pub fn binary<L, R>(left: L, operator: OperatorKind, right: R) -> Self where
//         L: Into<Expression>,
//         R: Into<Expression>,
//     {
//         Expression::Binary {
//             parenthesized: false,
//             operator: operator,
//             left: Box::new(left.into()),
//             right: Box::new(right.into()),
//         }
//     }

//     #[inline]
//     pub fn member<E, S>(object: E, property: S) -> Self where
//         E: Into<Expression>,
//         S: Into<Ident>
//     {
//         Expression::Member {
//             object: Box::new(object.into()),
//             property: property.into(),
//         }
//     }

//     #[inline]
//     pub fn call<E>(callee: E, arguments: Vec<Expression>) -> Self where
//         E: Into<Expression>
//     {
//         Expression::Call {
//             callee: Box::new(callee.into()),
//             arguments: arguments,
//         }
//     }

//     #[inline]
//     pub fn iefe(body: Vec<Statement>) -> Self {
//         Expression::call(Expression::Function {
//             name: None,
//             params: Vec::new(),
//             body: body,
//         }, Vec::new())
//     }

//     #[inline]
//     pub fn parenthesize(mut self) -> Self {
//         if let Expression::Binary {
//             ref mut parenthesized,
//             ..
//         } = self {
//             *parenthesized = true;
//         }

//         self
//     }

//     #[inline]
//     pub fn needs_parens(&self, bp: u8) -> bool {
//         match *self {
//             Expression::Binary {
//                 ref parenthesized,
//                 ref operator,
//                 ..
//             } => *parenthesized && bp >= operator.binding_power(),
//             _ => false
//         }
//     }

//     #[inline]
//     pub fn is_allowed_as_bare_statement(&self) -> bool {
//         match *self {
//             Expression::Object(_)       => false,
//             Expression::Function { .. } => false,

//             _                           => true,
//         }
//     }
// }

// impl From<Slice> for Expression {
//     #[inline]
//     fn from(slice: Slice) -> Expression {
//         Expression::Identifier(slice.into())
//     }
// }

// impl From<&'static str> for Expression {
//     #[inline]
//     fn from(string: &'static str) -> Expression {
//         Expression::Identifier(string.into())
//     }
// }

// impl From<Ident> for Expression {
//     #[inline]
//     fn from(ident: Ident) -> Self {
//         Expression::Identifier(ident)
//     }
// }

// // impl<I> From<I> for Expression where I: Into<Ident> {
// //     #[inline]
// //     fn from(ident: I) -> Self {
// //         Expression::Identifier(ident.into())
// //     }
// // }

// #[derive(Debug, PartialEq, Clone)]
// pub enum ObjectMember {
//     Shorthand {
//         key: Ident,
//     },
//     Value {
//         key: ObjectKey,
//         value: Expression,
//     },
//     Method {
//         key: ObjectKey,
//         params: Vec<Parameter>,
//         body: Vec<Statement>,
//     },
// }

// #[derive(Debug, PartialEq, Clone)]
// pub enum ObjectKey {
//     Computed(Expression),
//     Literal(Ident),
//     Binary(u64),
// }

// #[derive(Debug, PartialEq, Clone)]
// pub enum ClassMember {
//     Constructor {
//         params: Vec<Parameter>,
//         body: Vec<Statement>,
//     },
//     Method {
//         is_static: bool,
//         key: ClassKey,
//         params: Vec<Parameter>,
//         body: Vec<Statement>,
//     },
//     Property {
//         is_static: bool,
//         key: ClassKey,
//         value: Expression,
//     }
// }

// #[derive(Debug, PartialEq, Clone)]
// pub enum ClassKey {
//     Computed(Expression),
//     Literal(Ident),
//     Number(Slice),
//     Binary(u64),
// }

// impl ClassKey {
//     #[inline]
//     pub fn is_constructor(&self, source: &str) -> bool {
//         match *self {
//             ClassKey::Literal(ref name) => name.as_str(source) == "constructor",

//             _ => false
//         }
//     }
// }

// #[derive(Debug, PartialEq, Clone, Copy)]
// pub enum VariableDeclarationKind {
//     Var,
//     Let,
//     Const,
// }

// #[derive(Debug, PartialEq, Clone)]
// pub struct VariableDeclarator {
//     pub name: Ident,
//     pub value: Option<Expression>,
// }

// #[derive(Debug, PartialEq, Clone)]
// pub enum Statement {
//     Empty,
//     Block {
//         body: Vec<Statement>,
//     },
//     // `Transparent` is not part of the language grammar, just a helper that
//     // allows the transformer to replace a single statement with multiple
//     // statements without messing with parent array.
//     Transparent {
//         body: Vec<Statement>,
//     },
//     Labeled {
//         label: Slice,
//         body: Box<Statement>,
//     },
//     VariableDeclaration {
//         kind: VariableDeclarationKind,
//         declarators: Vec<VariableDeclarator>,
//     },
//     Expression {
//         value: Expression
//     },
//     Return {
//         value: Option<Expression>,
//     },
//     Break {
//         label: Option<Slice>,
//     },
//     Function {
//         name: Ident,
//         params: Vec<Parameter>,
//         body: Vec<Statement>,
//     },
//     Class {
//         name: Ident,
//         extends: Option<Ident>,
//         body: Vec<ClassMember>,
//     },
//     If {
//         test: Expression,
//         consequent: Box<Statement>,
//         alternate: Option<Box<Statement>>,
//     },
//     While {
//         test: Expression,
//         body: Box<Statement>,
//     },
//     Do {
//         test: Expression,
//         body: Box<Statement>
//     },
//     For {
//         init: Option<Box<Statement>>,
//         test: Option<Expression>,
//         update: Option<Expression>,
//         body: Box<Statement>,
//     },
//     ForIn {
//         left: Box<Statement>,
//         right: Expression,
//         body: Box<Statement>,
//     },
//     ForOf {
//         left: Box<Statement>,
//         right: Expression,
//         body: Box<Statement>,
//     },
//     Throw {
//         value: Expression
//     },
//     Try {
//         body: Box<Statement>,
//         error: Ident,
//         handler: Box<Statement>,
//     }
// }

// impl From<Expression> for Statement {
//     #[inline]
//     fn from(expression: Expression) -> Self {
//         Statement::Expression {
//             value: expression
//         }
//     }
// }

// #[derive(Debug, PartialEq)]
// pub struct Program {
//     pub source: String,
//     pub body: Vec<Statement>,
// }
