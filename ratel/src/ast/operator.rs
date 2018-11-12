use lexer::Token;
use lexer::Token::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperatorKind {
    FatArrow,         //   …  => …
    New,              //     new …
    Increment,        //      ++ … | … ++
    Decrement,        //      -- … | … --
    LogicalNot,       //       ! …
    BitwiseNot,       //       ~ …
    Typeof,           //  typeof …
    Void,             //    void …
    Delete,           //  delete …
    Multiplication,   //   …  *  …
    Division,         //   …  /  …
    Remainder,        //   …  %  …
    Exponent,         //   …  ** …
    Addition,         //   …  +  … | + …
    Subtraction,      //   …  -  … | - …
    BitShiftLeft,     //   …  << …
    BitShiftRight,    //   …  >> …
    UBitShiftRight,   //   … >>> …
    Lesser,           //   …  <  …
    LesserEquals,     //   …  <= …
    Greater,          //   …  >  …
    GreaterEquals,    //   …  >= …
    Instanceof,       //   … instanceof …
    In,               //   …  in …
    StrictEquality,   //   … === …
    StrictInequality, //   … !== …
    Equality,         //   …  == …
    Inequality,       //   …  != …
    BitwiseAnd,       //   …  &  …
    BitwiseXor,       //   …  ^  …
    BitwiseOr,        //   …  |  …
    LogicalAnd,       //   …  && …
    LogicalOr,        //   …  || …
    Conditional,      //   …  ?  …  :  …
    Assign,           //   …  =  …
    AddAssign,        //   …  += …
    SubtractAssign,   //   …  -= …
    ExponentAssign,   //   … **= …
    MultiplyAssign,   //   …  *= …
    DivideAssign,     //   …  /= …
    RemainderAssign,  //   …  %= …
    BSLAssign,        //   … <<= …
    BSRAssign,        //   … >>= …
    UBSRAssign,       //   … >>>= …
    BitAndAssign,     //   …  &= …
    BitXorAssign,     //   …  ^= …
    BitOrAssign,      //   …  |= …
    Spread,           //     ... …
}


#[derive(Debug, PartialEq, Clone)]
pub enum OperatorCategory {
    Minus,
    Plus,
    Safe,
    Word,
}

use self::OperatorKind::*;

impl OperatorKind {
    #[inline]
    pub fn from_token(token: Token) -> Option<OperatorKind> {
        match token {
            OperatorFatArrow         => Some(FatArrow),
            OperatorNew              => Some(New),
            OperatorIncrement        => Some(Increment),
            OperatorDecrement        => Some(Decrement),
            OperatorLogicalNot       => Some(LogicalNot),
            OperatorBitwiseNot       => Some(BitwiseNot),
            OperatorTypeof           => Some(Typeof),
            OperatorVoid             => Some(Void),
            OperatorDelete           => Some(Delete),
            OperatorMultiplication   => Some(Multiplication),
            OperatorDivision         => Some(Division),
            OperatorRemainder        => Some(Remainder),
            OperatorExponent         => Some(Exponent),
            OperatorAddition         => Some(Addition),
            OperatorSubtraction      => Some(Subtraction),
            OperatorBitShiftLeft     => Some(BitShiftLeft),
            OperatorBitShiftRight    => Some(BitShiftRight),
            OperatorUBitShiftRight   => Some(UBitShiftRight),
            OperatorLesser           => Some(Lesser),
            OperatorLesserEquals     => Some(LesserEquals),
            OperatorGreater          => Some(Greater),
            OperatorGreaterEquals    => Some(GreaterEquals),
            OperatorInstanceof       => Some(Instanceof),
            OperatorIn               => Some(In),
            OperatorStrictEquality   => Some(StrictEquality),
            OperatorStrictInequality => Some(StrictInequality),
            OperatorEquality         => Some(Equality),
            OperatorInequality       => Some(Inequality),
            OperatorBitwiseAnd       => Some(BitwiseAnd),
            OperatorBitwiseXor       => Some(BitwiseXor),
            OperatorBitwiseOr        => Some(BitwiseOr),
            OperatorLogicalAnd       => Some(LogicalAnd),
            OperatorLogicalOr        => Some(LogicalOr),
            OperatorConditional      => Some(Conditional),
            OperatorAssign           => Some(Assign),
            OperatorAddAssign        => Some(AddAssign),
            OperatorSubtractAssign   => Some(SubtractAssign),
            OperatorExponentAssign   => Some(ExponentAssign),
            OperatorMultiplyAssign   => Some(MultiplyAssign),
            OperatorDivideAssign     => Some(DivideAssign),
            OperatorRemainderAssign  => Some(RemainderAssign),
            OperatorBSLAssign        => Some(BSLAssign),
            OperatorBSRAssign        => Some(BSRAssign),
            OperatorUBSRAssign       => Some(UBSRAssign),
            OperatorBitAndAssign     => Some(BitAndAssign),
            OperatorBitXorAssign     => Some(BitXorAssign),
            OperatorBitOrAssign      => Some(BitOrAssign),
            OperatorSpread           => Some(Spread),
            _                        => None
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            FatArrow         => "=>",
            New              => "new",
            Increment        => "++",
            Decrement        => "--",
            LogicalNot       => "!",
            BitwiseNot       => "~",
            Typeof           => "typeof",
            Void             => "void",
            Delete           => "delete",
            Multiplication   => "*",
            Division         => "/",
            Remainder        => "%",
            Exponent         => "**",
            Addition         => "+",
            Subtraction      => "-",
            BitShiftLeft     => "<<",
            BitShiftRight    => ">>",
            UBitShiftRight   => ">>>",
            Lesser           => "<",
            LesserEquals     => "<=",
            Greater          => ">",
            GreaterEquals    => ">=",
            Instanceof       => "instanceof",
            In               => "in",
            StrictEquality   => "===",
            StrictInequality => "!==",
            Equality         => "==",
            Inequality       => "!=",
            BitwiseAnd       => "&",
            BitwiseXor       => "^",
            BitwiseOr        => "|",
            LogicalAnd       => "&&",
            LogicalOr        => "||",
            Conditional      => "?",
            Assign           => "=",
            AddAssign        => "+=",
            SubtractAssign   => "-=",
            ExponentAssign   => "**=",
            MultiplyAssign   => "*=",
            DivideAssign     => "/=",
            RemainderAssign  => "%=",
            BSLAssign        => "<<=",
            BSRAssign        => ">>=",
            UBSRAssign       => ">>>=",
            BitAndAssign     => "&=",
            BitXorAssign     => "^=",
            BitOrAssign      => "|=",
            Spread           => "...",
        }
    }

    /// According to the Operator Precedence Table
    /// Note: Unary operators default to 15!
    #[inline]
    pub fn binding_power(&self) -> u8 {
        match self {
            FatArrow         => 18,

            New              => 17,

            Increment        |
            Decrement        => 16,

            LogicalNot       |
            BitwiseNot       |
            Typeof           |
            Void             |
            Delete           => 15,

            Multiplication   |
            Division         |
            Remainder        |
            Exponent         => 14,

            Addition         |
            Subtraction      => 13,

            BitShiftLeft     |
            BitShiftRight    |
            UBitShiftRight   => 12,

            Lesser           |
            LesserEquals     |
            Greater          |
            GreaterEquals    |
            Instanceof       |
            In               => 11,

            StrictEquality   |
            StrictInequality |
            Equality         |
            Inequality       => 10,

            BitwiseAnd       => 9,
            BitwiseXor       => 8,
            BitwiseOr        => 7,
            LogicalAnd       => 6,
            LogicalOr        => 5,
            Conditional      => 4,

            Assign           |
            AddAssign        |
            SubtractAssign   |
            ExponentAssign   |
            MultiplyAssign   |
            DivideAssign     |
            RemainderAssign  |
            BSLAssign        |
            BSRAssign        |
            UBSRAssign       |
            BitAndAssign     |
            BitXorAssign     |
            BitOrAssign      => 3,

            Spread           => 1,
        }
    }

    #[inline]
    pub fn prefix(&self) -> bool {
        match self {
            LogicalNot       |
            BitwiseNot       |
            Typeof           |
            Void             |
            Delete           |
            New              |
            Spread           |
            Increment        |
            Decrement        |
            Addition         |
            Subtraction      => true,

            _                => false
        }
    }

    #[inline]
    pub fn infix(&self) -> bool {
        match self {
            FatArrow         |
            Multiplication   |
            Division         |
            Remainder        |
            Exponent         |
            StrictEquality   |
            StrictInequality |
            Equality         |
            Inequality       |
            Lesser           |
            LesserEquals     |
            Greater          |
            GreaterEquals    |
            Instanceof       |
            In               |
            BitShiftLeft     |
            BitShiftRight    |
            UBitShiftRight   |
            BitwiseAnd       |
            BitwiseXor       |
            BitwiseOr        |
            LogicalAnd       |
            LogicalOr        |
            Conditional      |
            Addition         |
            Subtraction      |
            Assign           |
            AddAssign        |
            SubtractAssign   |
            ExponentAssign   |
            MultiplyAssign   |
            DivideAssign     |
            RemainderAssign  |
            BSLAssign        |
            BSRAssign        |
            UBSRAssign       |
            BitAndAssign     |
            BitXorAssign     |
            BitOrAssign      => true,

            _                => false
        }
    }

    #[inline]
    pub fn assignment(&self) -> bool {
        match self {
            Assign           |
            AddAssign        |
            SubtractAssign   |
            ExponentAssign   |
            MultiplyAssign   |
            DivideAssign     |
            RemainderAssign  |
            BSLAssign        |
            BSRAssign        |
            UBSRAssign       |
            BitAndAssign     |
            BitXorAssign     |
            BitOrAssign      => true,

            _                => false
        }
    }

    #[inline]
    pub fn category(&self) -> OperatorCategory {
        match self {
            In            |
            New           |
            Typeof        |
            Void          |
            Delete        |
            Instanceof    => OperatorCategory::Word,
            Addition      |
            Increment     => OperatorCategory::Plus,
            Subtraction   |
            Decrement     => OperatorCategory::Minus,

            _             => OperatorCategory::Safe,
        }
    }
}
