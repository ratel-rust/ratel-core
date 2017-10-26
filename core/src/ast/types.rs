use ast::{Loc, TypePtr, TypeList};

static TYPE_MIXED: &Loc<Type<'static>> = &Loc {
    start: 0,
    end: 0,
    item: Type::Mixed
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
    Mixed,
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
