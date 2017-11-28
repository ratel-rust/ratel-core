use ast::{Loc, TypeNode, TypeList};

static TYPE_ANY: &Loc<Type<'static>> = &Loc {
    start: 0,
    end: 0,
    item: Type::Any
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Primitive {
    Number,
    String,
    Boolean,
    Null,
    Undefined,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type<'ast> {
    Any,
    Primitive(Primitive),
    Identifier(&'ast str),
    Union {
        variants: TypeList<'ast>,
    },
    Generic {
        ident: &'ast str,
        subtypes: TypeList<'ast>,
    }
}
