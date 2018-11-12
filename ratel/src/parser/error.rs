use error::Error;

use ast::{Node, Loc, NodeList, Pattern};
use ast::{Name, ClassMember, Property, PropertyKey, MandatoryName, Block};
use parser::Parser;

pub trait Handle<'ast> {
    fn handle_error(parser: &mut Parser<'ast>, err: Error) -> Self;
}

pub trait ToError {
    fn to_error() -> Self;
}

impl<'ast, I> ToError for Block<'ast, I> {
    fn to_error() -> Self {
        Block { body: NodeList::empty() }
    }
}

impl<'ast> ToError for MandatoryName<'ast> {
    fn to_error() -> Self {
        MandatoryName::empty()
    }
}

impl<'ast> ToError for Node<'ast, Property<'ast>> {
    fn to_error() -> Self {
        Node::new(&Loc {
            start: 0,
            end: 0,
            item: Property::Shorthand("")
        })
    }
}

impl<'ast> ToError for Node<'ast, ClassMember<'ast>> {
    fn to_error() -> Self {
        Node::new(&Loc {
            start: 0,
            end: 0,
            item: ClassMember::Error,
        })
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

impl<'ast, T: 'ast + Copy> ToError for NodeList<'ast, T> {
    fn to_error() -> Self {
        NodeList::empty()
    }
}

impl<'ast> ToError for Node<'ast, &'ast str> {
    fn to_error() -> Self {
        Node::new(&Loc {
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

impl<'ast> ToError for Node<'ast, Pattern<'ast>> {
    #[inline]
    fn to_error() -> Self {
        Node::new(&Loc {
            start: 0,
            end: 0,
            item: Pattern::Void
        })
    }
}

impl<'ast> ToError for Node<'ast, PropertyKey<'ast>> {
    #[inline]
    fn to_error() -> Self {
        Node::new(&Loc {
            start: 0,
            end: 0,
            item: PropertyKey::Literal("")
        })
    }
}

impl ToError for () {
    #[inline]
    fn to_error() {}
}
