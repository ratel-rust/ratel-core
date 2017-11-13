// Lookup table layout:
// ====================
//
// EOF   ;     :     ,     (     )     [     ]     {     }     =>    NEW
// ++    --    !     ~     TYPOF VOID  DELET *     /     %     **    +
// -     <<    >>    >>>   <     <=    >     >=    INSOF IN    ===   !==
// ==    !=    &     ^     |     &&    ||    ?     =     +=    -=    **=
// *=    /=    %=    <<=   >>=   >>>=  &=    ^=    |=    ...   VAR   LET
// CONST BREAK DO    CASE  ELSE  CATCH EXPRT CLASS EXTND RET   WHILE FINLY
// SUPER WITH  CONT  FOR   SWTCH YIELD DBGGR FUNCT THIS  DEFLT IF    THROW
// IMPRT TRY   STATI TRUE  FALSE NULL  UNDEF STR   NUM   BIN   REGEX ENUM
// IMPL  PCKG  PROT  IFACE PRIV  PUBLI IDENT ACCSS TPL_O TPL_C ERR_T ERR_E

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
    OperatorFatArrow,         //   …  => …
    OperatorNew,              //     new …
    OperatorIncrement,        //      ++ … | … ++
    OperatorDecrement,        //      -- … | … --
    OperatorLogicalNot,       //       ! …
    OperatorBitwiseNot,       //       ~ …
    OperatorTypeof,           //  typeof …
    OperatorVoid,             //    void …
    OperatorDelete,           //  delete …
    OperatorMultiplication,   //   …  *  …
    OperatorDivision,         //   …  /  …
    OperatorRemainder,        //   …  %  …
    OperatorExponent,         //   …  ** …
    OperatorAddition,         //   …  +  … | + …
    OperatorSubtraction,     //   …  -  … | - …
    OperatorBitShiftLeft,     //   …  << …
    OperatorBitShiftRight,    //   …  >> …
    OperatorUBitShiftRight,   //   … >>> …
    OperatorLesser,           //   …  <  …
    OperatorLesserEquals,     //   …  <= …
    OperatorGreater,          //   …  >  …
    OperatorGreaterEquals,    //   …  >= …
    OperatorInstanceof,       //   … instanceof …
    OperatorIn,               //   …  in …
    OperatorStrictEquality,   //   … === …
    OperatorStrictInequality, //   … !== …
    OperatorEquality,         //   …  == …
    OperatorInequality,       //   …  != …
    OperatorBitwiseAnd,       //   …  &  …
    OperatorBitwiseXor,       //   …  ^  …
    OperatorBitwiseOr,        //   …  |  …
    OperatorLogicalAnd,       //   …  && …
    OperatorLogicalOr,        //   …  || …
    OperatorConditional,      //   …  ?  …  :  …
    OperatorAssign,           //   …  =  …
    OperatorAddAssign,        //   …  += …
    OperatorSubtractAssign,  //   …  -= …
    OperatorExponentAssign,   //   … **= …
    OperatorMultiplyAssign,   //   …  *= …
    OperatorDivideAssign,     //   …  /= …
    OperatorRemainderAssign,  //   …  %= …
    OperatorBSLAssign,        //   … <<= …
    OperatorBSRAssign,        //   … >>= …
    OperatorUBSRAssign,       //   … >>>= …
    OperatorBitAndAssign,     //   …  &= …
    OperatorBitXorAssign,     //   …  ^= …
    OperatorBitOrAssign,      //   …  |= …
    OperatorSpread,           //     ... …
    DeclarationVar,
    DeclarationLet,
    DeclarationConst,
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
    LiteralTrue,
    LiteralFalse,
    LiteralNull,
    LiteralUndefined,
    LiteralString,
    LiteralNumber,
    LiteralBinary,
    LiteralRegEx,
    ReservedEnum,
    ReservedImplements,
    ReservedPackage,
    ReservedProtected,
    ReservedInterface,
    ReservedPrivate,
    ReservedPublic,
    Identifier,
    Accessor,
    TemplateOpen,
    TemplateClosed,
    UnexpectedToken,
    UnexpectedEndOfProgram,
}

impl Token {
    pub fn as_word(&self) -> Option<&'static str> {
        use self::Token::*;

        match *self {
            Break              => Some("break"),
            Do                 => Some("do"),
            Case               => Some("case"),
            Else               => Some("else"),
            Catch              => Some("catch"),
            Export             => Some("export"),
            Class              => Some("class"),
            Extends            => Some("extends"),
            Return             => Some("return"),
            While              => Some("while"),
            Finally            => Some("finally"),
            Super              => Some("super"),
            With               => Some("with"),
            Continue           => Some("continue"),
            For                => Some("for"),
            Switch             => Some("switch"),
            Yield              => Some("yield"),
            Debugger           => Some("debugger"),
            Function           => Some("function"),
            This               => Some("this"),
            Default            => Some("default"),
            If                 => Some("if"),
            Throw              => Some("throw"),
            Import             => Some("import"),
            Try                => Some("try"),
            Static             => Some("static"),
            OperatorNew        => Some("new"),
            OperatorTypeof     => Some("typeof"),
            OperatorVoid       => Some("void"),
            OperatorDelete     => Some("delete"),
            OperatorInstanceof => Some("instanceof"),
            LiteralTrue        => Some("true"),
            LiteralFalse       => Some("false"),
            LiteralNull        => Some("null"),
            LiteralUndefined   => Some("undefined"),

            _                  => None,
        }
    }
}
