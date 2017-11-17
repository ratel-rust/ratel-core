use parser::Parser;
use lexer::Token;
use lexer::Token::*;
use ast::{Loc, List, ListBuilder, OperatorKind, Expression, ExpressionPtr};
use ast::expression::{SequenceExpr, MemberExpr, ComputedMemberExpr, CallExpr, BinaryExpr};
use ast::expression::{PostfixExpr, ConditionalExpr};
use ast::OperatorKind::*;


type NestedHandler = Option<for<'ast> fn(&mut Parser<'ast>, ExpressionPtr<'ast>) -> ExpressionPtr<'ast>>;
type Lookup = &'static [NestedHandler; 108];

/// All potential tokens, including Comma for sequence expressions
pub static B0: Lookup = &[
    ____, ____, ____, SEQ,  CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
//  EOF   ;     :     ,     (     )     [     ]     {     }     =>    NEW

    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
//  ++    --    !     ~     TYPOF VOID  DELET *     /     %     **    +

    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
//  -     <<    >>    >>>   <     <=    >     >=    INSOF IN    ===   !==

    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  OR,   COND, ASGN, ADDA, SUBA, EXPA,
//  ==    !=    &     ^     |     &&    ||    ?     =     +=    -=    **=

    MULA, DIVA, REMA, BSLA, BSRA, UBSA, BWAA, XORA, BORA, ____, ____, ____,
//  *=    /=    %=    <<=   >>=   >>>=  &=    ^=    |=    ...   VAR   LET

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  CONST BREAK DO    CASE  ELSE  CATCH EXPRT CLASS EXTND RET   WHILE FINLY

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  SUPER WITH  CONT  FOR   SWTCH YIELD DBGGR FUNCT THIS  DEFLT IF    THROW

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  IMPRT TRY   STATI TRUE  FALSE NULL  UNDEF STR   NUM   BIN   REGEX ENUM

    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
//  IMPL  PCKG  PROT  IFACE PRIV  PUBLI IDENT ACCSS TPL_O TPL_C ERR_T ERR_E
];

pub static B1: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  OR,   COND, ASGN, ADDA, SUBA, EXPA,
    MULA, DIVA, REMA, BSLA, BSRA, UBSA, BWAA, XORA, BORA, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B4: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  OR,   COND, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B5: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  OR,   ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B6: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B7: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B8: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B9: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B10: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B11: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B12: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B13: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B14: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

pub static B15: Lookup = &[
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, ____, ____, ____, EXPN, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

// pub static B17: Lookup = &[
//     ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
//     INC,  DEC,  ____, ____, ____, ____, ____, ____, ____, ____, EXPN, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
// ];

// pub static B18: Lookup = &[
//     ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
//     INC,  DEC,  ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
// ];

// pub static B19: Lookup = &[
//     ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//     ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
// ];

const ____: NestedHandler = None;

const SEQ: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let mut builder = ListBuilder::new(par.arena, left);
    builder.push(par.expression(B1));

    while let Comma = par.lexer.token {
        par.lexer.consume();
        builder.push(par.expression(B1));
    }

    par.alloc(Expression::Sequence(SequenceExpr {
        body: builder.into_list()
    }).at(0, 0))
});

const INC: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    // TODO: op.end
    par.alloc(Loc::new(left.start, left.end, Expression::Postfix(PostfixExpr {
        operator: OperatorKind::Increment,
        operand: left,
    })))
});

const DEC: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    // TODO: op.end
    par.alloc(Loc::new(left.start, left.end, Expression::Postfix(PostfixExpr {
        operator: OperatorKind::Decrement,
        operand: left,
    })))
});

const COND: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let consequent = par.expression(B4);
    expect!(par, Colon);
    let alternate = par.expression(B4);

    par.alloc(Expression::Conditional(ConditionalExpr {
        test: left,
        consequent: consequent,
        alternate: alternate,
    }).at(0, 0))
});

const ARRW: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let params = match left.item {
        Expression::Sequence(SequenceExpr { body }) => body,
        _                                           => List::from(par.arena, left)
    };

    return par.arrow_function_expression(params);
});

const ACCS: NestedHandler = Some(|par, left| {
    let member = par.lexer.accessor_as_str();
    par.lexer.consume();

    let right = par.alloc_in_loc(member);

    par.alloc(Loc::new(left.start, right.end, Expression::Member(MemberExpr {
        object: left,
        property: right,
    })))
});

const CALL: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let arguments = par.expression_list();

    par.alloc(Expression::Call(CallExpr {
        callee: left,
        arguments,
    }).at(0, 0))
});

const CMEM: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let property = par.expression(B0);

    expect!(par, BracketClose);

    par.alloc(Expression::ComputedMember(ComputedMemberExpr {
        object: left,
        property: property,
    }).at(0, 0))
});

const TPLS: NestedHandler = Some(|par, left| {
    par.template_string(Some(left))
});

const TPLE: NestedHandler = Some(|par, left| {
    par.template_expression(Some(left))
});

macro_rules! binary {
    ($name:ident, $bp:expr => $op:ident) => {
        const $name: NestedHandler = {
            fn handler<'ast>(par: &mut Parser<'ast>, left: ExpressionPtr<'ast>) -> ExpressionPtr<'ast> {
                par.lexer.consume();

                let right = par.expression($bp);

                par.alloc(Loc::new(left.start, right.end, Expression::Binary(BinaryExpr {
                    operator: $op,
                    left,
                    right,
                })))
            }

            Some(handler)
        };
    }
}

binary!(ASGN , B1  => Assign);
binary!(ADDA , B1  => AddAssign);
binary!(SUBA , B1  => SubtractAssign);
binary!(EXPA , B1  => ExponentAssign);
binary!(MULA , B1  => MultiplyAssign);
binary!(DIVA , B1  => DivideAssign);
binary!(REMA , B1  => RemainderAssign);
binary!(BSLA , B1  => BSLAssign);
binary!(BSRA , B1  => BSRAssign);
binary!(UBSA , B1  => UBSRAssign);
binary!(BWAA , B1  => BitAndAssign);
binary!(XORA , B1  => BitXorAssign);
binary!(BORA , B1  => BitOrAssign);
binary!(OR   , B5  => LogicalOr);
binary!(AND  , B6  => LogicalAnd);
binary!(BWOR , B7  => BitwiseOr);
binary!(BWXO , B8  => BitwiseXor);
binary!(BWAN , B9  => BitwiseAnd);
binary!(STEQ , B10 => StrictEquality);
binary!(SIEQ , B10 => StrictInequality);
binary!(EQ   , B10 => Equality);
binary!(INEQ , B10 => Inequality);
binary!(LESS , B11 => Lesser);
binary!(LSEQ , B11 => LesserEquals);
binary!(GRTR , B11 => Greater);
binary!(GREQ , B11 => GreaterEquals);
binary!(INOF , B11 => Instanceof);
binary!(IN   , B11 => In);
binary!(BSL  , B12 => BitShiftLeft);
binary!(BSR  , B12 => BitShiftRight);
binary!(UBSR , B12 => UBitShiftRight);
binary!(ADD  , B13 => Addition);
binary!(SUB  , B13 => Subtraction);
binary!(MUL  , B14 => Multiplication);
binary!(DIV  , B14 => Division);
binary!(REM  , B14 => Remainder);
binary!(EXPN , B15 => Exponent);


impl<'ast> Parser<'ast> {
    #[inline]
    pub fn expression(&mut self, lookup: Lookup) -> ExpressionPtr<'ast> {
        let left = self.bound_expression();

        self.nested_expression(left, lookup)
    }

    #[inline]
    pub fn nested_expression(&mut self, mut left: ExpressionPtr<'ast>, lookup: Lookup) -> ExpressionPtr<'ast> {
        loop {
            left = match lookup[self.lexer.token as usize] {
                Some(handler) => handler(self, left),
                None          => return left,
            }
        }
    }
}
