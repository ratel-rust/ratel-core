use error::Error;

use ast::{Ptr, Loc, List, Statement, StatementPtr, Expression, ExpressionPtr};
use ast::{Declarator, DeclaratorId, ObjectMember, Parameter, ParameterKey, ParameterPtr};
use ast::{Name, Function, Class, ClassMember, MandatoryName, Block};
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

impl<'ast, I> ToError for Ptr<'ast, Loc<Block<'ast, I>>> {
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

impl<'ast> ToError for ParameterPtr<'ast> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: Parameter {
                key: ParameterKey::Identifier(""),
                value: None
            }
        })
    }
}

impl<'ast> ToError for Ptr<'ast, Loc<Declarator<'ast>>> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: Declarator {
                name: DeclaratorId::Identifier(""),
                value: None
            }
        })
    }
}

impl<'ast> ToError for ObjectMember<'ast> {
    fn to_error() -> Self {
        ObjectMember::Shorthand("")
    }
}

impl<'ast> ToError for Ptr<'ast, Loc<ObjectMember<'ast>>> {
    fn to_error() -> Self {
        Ptr::new(&Loc {
            start: 0,
            end: 0,
            item: ObjectMember::Shorthand("")
        })
    }
}

impl<'ast> ToError for Ptr<'ast, Loc<ClassMember<'ast>>> {
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

impl<'ast, T: 'ast + Copy> ToError for List<'ast, Loc<T>> {
    fn to_error() -> Self {
        List::empty()
    }
}
