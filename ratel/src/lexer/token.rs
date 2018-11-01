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

macro_rules! count {
    () => (0);
    ( $x:tt $($xs:tt)* ) => (1 + count!($($xs)*));
}

macro_rules! token {
    ($( $name:ident, )*) => {
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum Token {
            $( $name, )*
        }

        const TOKEN_SIZE: usize = count!($( $name )*);
    }
}

token! {
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
    OperatorSubtraction,      //   …  -  … | - …
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

#[derive(Clone, Copy)]
pub struct TokenTable<T>([T; TOKEN_SIZE]);

impl<T> TokenTable<T>
where
    T: Copy
{
    pub fn set(&mut self, index: Token, value: T) {
        self.0[index as usize] = value;
    }

    pub fn get(&self, index: Token) -> T {
        self.0[index as usize]
    }

    pub fn extend(&mut self, inserts: &[(Token, T)]) {
        for (token, value) in inserts {
            self.set(*token, *value);
        }
    }

    pub fn extend_copy(&self, inserts: &[(Token, T)]) -> TokenTable<T> {
        let mut table = *self;
        table.extend(inserts);
        table
    }
}

impl<T> Default for TokenTable<Option<T>>
where
    T: Copy
{
    fn default() -> TokenTable<Option<T>> {
        TokenTable([None; TOKEN_SIZE])
    }
}

impl Token {
    pub fn size() -> usize {
        TOKEN_SIZE
    }

    pub fn table<T>(fill: T, inserts: &[(Token, T)]) -> TokenTable<T>
    where
        T: Copy,
    {
        let mut table = TokenTable([fill; TOKEN_SIZE]);

        for (token, value) in inserts {
            table.set(*token, *value);
        }

        table
    }

    pub fn is_word(&self) -> bool {
        use self::Token::*;

        match *self {
            Identifier         |
            Break              |
            Do                 |
            Case               |
            Else               |
            Catch              |
            Export             |
            Class              |
            Extends            |
            Return             |
            While              |
            Finally            |
            Super              |
            With               |
            Continue           |
            For                |
            Switch             |
            Yield              |
            Debugger           |
            Function           |
            This               |
            Default            |
            If                 |
            Throw              |
            Import             |
            Try                |
            Static             |
            OperatorNew        |
            OperatorTypeof     |
            OperatorVoid       |
            OperatorDelete     |
            OperatorInstanceof |
            LiteralTrue        |
            LiteralFalse       |
            LiteralNull        |
            LiteralUndefined   => true,

            _                  => false,
        }
    }
}
