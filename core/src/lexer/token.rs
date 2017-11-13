use ast::{Value, OperatorKind, DeclarationKind};

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
pub enum TemplateKind<'src> {
    Open(&'src str),
    Closed(&'src str),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'src> {
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
    Declaration(DeclarationKind),
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
    Literal(Value<'src>),
    Reserved(ReservedKind),
    Identifier,
    Accessor(&'src str),
    Template(TemplateKind<'src>),
    UnexpectedToken,
    UnexpectedEndOfProgram,
}

impl<'src> Token<'src> {
    pub fn as_word(&self) -> Option<&'static str> {
        use self::Token::*;
        use ast::OperatorKind::*;
        use ast::Value::*;

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
