#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Value<'ast> {
    Undefined,
    Null,
    True,
    False,
    Number(&'ast str),
    Binary(&'ast str),
    String(&'ast str),
    RawQuasi(&'ast str),
    RegEx(&'ast str),
}
