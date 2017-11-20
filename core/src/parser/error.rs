use error::Error;

use ast::{Ptr, Loc, List, Statement, StatementPtr, Expression, ExpressionPtr, Pattern};
use ast::{Name, Function, Class, ClassMember, Property, PropertyKey, MandatoryName, Block};
use parser::Parser;

pub trait Handle<'ast> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self;
}

pub trait ToError {
    fn to_error() -> Self;
}

impl<'ast> ToError for Statement<'ast> {
    fn to_error() -> Self {
        Statement::Error
    }
}

impl<'ast> ToError for StatementPtr<'ast> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: Statement::Error
        })
    }
}

impl<'ast, I> ToError for Block<'ast, I> {
    fn to_error() -> Self {
        Block { body: List::empty() }
    }
}

impl<'ast, I> ToError for Ptr<'ast, Block<'ast, I>> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: Block { body: empty_list!() }
        })
    }
}

impl<'ast> ToError for Expression<'ast> {
    fn to_error() -> Self {
        Expression::Error
    }
}

impl<'ast> ToError for ExpressionPtr<'ast> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: Expression::Error
        })
    }
}

impl<'ast> ToError for MandatoryName<'ast> {
    fn to_error() -> Self {
        MandatoryName::empty()
    }
}

impl<'ast> ToError for Ptr<'ast, Property<'ast>> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: Property::Shorthand("")
        })
    }
}

impl<'ast> ToError for Ptr<'ast, ClassMember<'ast>> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: ClassMember::Error,
        })
    }
}

impl<'ast, N: Name<'ast>> ToError for Function<'ast, N> {
    fn to_error() -> Self {
        Function {
            name: N::empty(),
            params: List::empty(),
            body: Ptr::new(&Loc {
                start: 0,
                end: 0,
                item: Block { body: empty_list!() }
            }),
        }
    }
}

impl<'ast, N: Name<'ast>> ToError for Class<'ast, N> {
    fn to_error() -> Self {
        Class {
            name: N::empty(),
            extends: None,
            body: Ptr::new(&Loc {
                start: 0,
                end: 0,
                item: Block { body: empty_list!() }
            }),
        }
    }
}

impl<'ast, T: 'ast + ToError> ToError for Loc<T> {
    fn to_error() -> Self {
        Loc {
            start: 0,
            end: 0,
            item: T::to_error()
        }
    }
}

impl<'ast, T: 'ast + Copy> ToError for List<'ast, T> {
    fn to_error() -> Self {
        List::empty()
    }
}

impl<'ast> ToError for Ptr<'ast, &'ast str> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: ""
        })
    }
}


impl<'ast> ToError for Pattern<'ast> {
    #[inline]
    fn to_error() -> Self {
        Pattern::Void
    }
}

impl<'ast> ToError for Ptr<'ast, Pattern<'ast>> {
    #[inline]
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: Pattern::Void
        })
    }
}

impl<'ast> ToError for Ptr<'ast, PropertyKey<'ast>> {
    #[inline]
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: PropertyKey::Literal("")
        })
    }
}

impl ToError for () {
    #[inline]
    fn to_error() -> Self {
        ()
    }
}
