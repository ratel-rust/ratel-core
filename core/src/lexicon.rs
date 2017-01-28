use grammar::VariableDeclarationKind;
use operator::OperatorKind;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Slice {
    start: usize,
    end: usize,
}

impl Slice {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Slice {
            start: start,
            end: end,
        }
    }

    #[inline]
    pub fn as_str(&self, src: &str) -> &str {
        &src[self.start..self.end]
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReservedKind {
    Enum,
    Implements,
    Package,
    Protected,
    Interface,
    Private,
    Public,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TemplateKind {
    Open(Slice),
    Closed(Slice),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token {
    EndOfProgram,
    Semicolon,
    Colon,
    Comma,
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    Operator(OperatorKind),
    Declaration(VariableDeclarationKind),
    Break,
    Do,
    Case,
    Else,
    Catch,
    Export,
    Class,
    Extends,
    Return,
    While,
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
    Import,
    Try,
    Static,
    Undefined,
    Null,
    LitBoolean(bool),
    LitNumber(Slice),
    LitBinary(u64),
    LitString(Slice),
    LitQuasi(Slice),
    Reserved(ReservedKind),
    Identifier(Slice),
    Template(TemplateKind),
}

impl Token {
    pub fn as_word(&self) -> Option<&'static str> {
        use self::Token::*;
        use operator::OperatorKind::*;
        use grammar::Value::*;

        match *self {
            Break                => Some("break"),
            Do                   => Some("do"),
            Case                 => Some("case"),
            Else                 => Some("else"),
            Catch                => Some("catch"),
            Export               => Some("export"),
            Class                => Some("class"),
            Extends              => Some("extends"),
            Return               => Some("return"),
            While                => Some("while"),
            Finally              => Some("finally"),
            Super                => Some("super"),
            With                 => Some("with"),
            Continue             => Some("continue"),
            For                  => Some("for"),
            Switch               => Some("switch"),
            Yield                => Some("yield"),
            Debugger             => Some("debugger"),
            Function             => Some("function"),
            This                 => Some("this"),
            Default              => Some("default"),
            If                   => Some("if"),
            Throw                => Some("throw"),
            Import               => Some("import"),
            Try                  => Some("try"),
            Static               => Some("static"),
            Operator(New)        => Some("new"),
            Operator(Typeof)     => Some("typeof"),
            Operator(Void)       => Some("void"),
            Operator(Delete)     => Some("delete"),
            Operator(Instanceof) => Some("instanceof"),
            Literal(True)        => Some("true"),
            Literal(False)       => Some("false"),
            Literal(Null)        => Some("null"),
            Literal(Undefined)   => Some("undefined"),

            _                    => None,
        }
    }
}
