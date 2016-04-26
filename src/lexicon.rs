use grammar::LiteralValue;

#[derive(Debug, PartialEq)]
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
    Await,
    Static,
}

#[derive(Debug, PartialEq)]
pub enum ReservedKind {
    Enum,
    Implements,
    Package,
    Protected,
    Interface,
    Private,
    Public,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum OperatorKind {
    Add,        // +
    Substract,  // -
    Multiply,   // *
    Divide,     // /
    Modulo,     // %
    Exponent,   // **
    Not,        // !
    Increment,  // ++
    Decrement,  // --
}

#[derive(Debug, PartialEq)]
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
    Literal(LiteralValue),
}
