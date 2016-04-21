#[derive(Debug)]
pub enum KeywordKind {
    Break,
    Do,
    In,
    Typeof,
    Case,
    Else,
    Instanceof,
    Var,
    Let,
    Catch,
    Export,
    New,
    Class,
    Extends,
    Return,
    While,
    Const,
    Finally,
    Super,
    With,
    Continue,
    For,
    Switch,
    Yield,
    Debugger,
    Function,
    This,
    Default,
    If,
    Throw,
    Delete,
    Import,
    Try,
    Void,
    Async,
    Await,
}

#[derive(Debug)]
pub enum ReservedKind {
    Enum,
    Implements,
    Package,
    Protected,
    Interface,
    Private,
    Public,
}

#[derive(Debug)]
pub enum CompareKind {
    Is,             // ===
    Isnt,           // !==
    Equals,         // ==
    NotEquals,      // !=
    Lesser,         // <
    LesserEquals,   // <=
    Greater,        // >
    GreaterEquals,  // >=
}

#[derive(Debug)]
pub enum OperatorKind {
    Add,        // +
    Substract,  // -
    Multiply,   // *
    Divide,     // /
    Modulo,     // %
    Exponent,   // **
    Not         // !
}

#[derive(Debug)]
pub enum Token {
    LineTermination,
    Semicolon,
    Comma,
    Colon,
    Accessor, // .
    Compare(CompareKind),
    Operator(OperatorKind),
    Assign,
    ParenOn,
    ParenOff,
    BracketOn,
    BracketOff,
    BlockOn,
    BlockOff,
    FatArrow, // =>
    Keyword(KeywordKind),
    Reserved(ReservedKind),
    Identifier(String),
    LiteralTrue,
    LiteralFalse,
    LiteralUndefined,
    LiteralNull,
    LiteralNumber(f64),
    LiteralString(String),
    Comment(String),
    BlockComment(String),
}
