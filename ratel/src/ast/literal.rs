#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal<'ast> {
    Undefined,
    Null,
    True,
    False,
    Number(&'ast str),
    Binary(&'ast str),
    String(&'ast str),
    RegEx(&'ast str),
}
