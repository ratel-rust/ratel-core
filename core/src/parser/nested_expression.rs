use parser::Parser;
use lexer::Token::*;
use ast::{Loc, List, ListBuilder, OperatorKind, Expression, ExpressionPtr};
use ast::OperatorKind::*;


type NestedHandler = Option<for<'ast> fn(&mut Parser<'ast>, ExpressionPtr<'ast>) -> ExpressionPtr<'ast>>;
type Lookup = &'static [NestedHandler; 108];

static BINDING_POWER: [Lookup; 20] = [
    &BINDING_POWER_0,
    &BINDING_POWER_1,
    &BINDING_POWER_1,
    &BINDING_POWER_1,
    &BINDING_POWER_4,
    &BINDING_POWER_5,
    &BINDING_POWER_6,
    &BINDING_POWER_7,
    &BINDING_POWER_8,
    &BINDING_POWER_9,
    &BINDING_POWER_10,
    &BINDING_POWER_11,
    &BINDING_POWER_12,
    &BINDING_POWER_13,
    &BINDING_POWER_14,
    &BINDING_POWER_15,
    &BINDING_POWER_15,
    &BINDING_POWER_17,
    &BINDING_POWER_18,
    &BINDING_POWER_19,
];

#[inline]
fn lookup(bp: u8) -> *const NestedHandler {
    unsafe {
        (*(&BINDING_POWER as *const Lookup).offset(bp as isize)) as *const NestedHandler
    }
}

/// All potential tokens, including Comma for sequence expressions
static BINDING_POWER_0: [NestedHandler; 108] = [
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

static BINDING_POWER_1: [NestedHandler; 108] = [
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

static BINDING_POWER_4: [NestedHandler; 108] = [
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

static BINDING_POWER_5: [NestedHandler; 108] = [
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

static BINDING_POWER_6: [NestedHandler; 108] = [
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

static BINDING_POWER_7: [NestedHandler; 108] = [
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

static BINDING_POWER_8: [NestedHandler; 108] = [
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

static BINDING_POWER_9: [NestedHandler; 108] = [
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

static BINDING_POWER_10: [NestedHandler; 108] = [
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

static BINDING_POWER_11: [NestedHandler; 108] = [
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

static BINDING_POWER_12: [NestedHandler; 108] = [
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

static BINDING_POWER_13: [NestedHandler; 108] = [
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

static BINDING_POWER_14: [NestedHandler; 108] = [
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

static BINDING_POWER_15: [NestedHandler; 108] = [
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

static BINDING_POWER_17: [NestedHandler; 108] = [
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

static BINDING_POWER_18: [NestedHandler; 108] = [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

static BINDING_POWER_19: [NestedHandler; 108] = [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
];

const ____: NestedHandler = None;

const SEQ: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let mut builder = ListBuilder::new(par.arena, left);
    builder.push(par.expression(1));

    while let Comma = par.lexer.token {
        par.lexer.consume();
        builder.push(par.expression(1));
    }

    par.alloc(Expression::Sequence {
        body: builder.into_list()
    }.at(0, 0))
});

const INC: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    // TODO: op.end
    par.alloc(Loc::new(left.start, left.end, Expression::Postfix {
        operator: OperatorKind::Increment,
        operand: left,
    }))
});

const DEC: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    // TODO: op.end
    par.alloc(Loc::new(left.start, left.end, Expression::Postfix {
        operator: OperatorKind::Decrement,
        operand: left,
    }))
});

const COND: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let consequent = par.expression(4);
    expect!(par, Colon);
    let alternate = par.expression(4);

    par.alloc(Expression::Conditional {
        test: left,
        consequent: consequent,
        alternate: alternate,
    }.at(0, 0))
});

const ARRW: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let params = match left.item {
        Expression::Sequence { body } => body,
        _                             => List::from(par.arena, left)
    };

    return par.arrow_function_expression(params);
});

const ACCS: NestedHandler = Some(|par, left| {
    let member = par.lexer.accessor_as_str();
    par.lexer.consume();

    let right = par.alloc_in_loc(member);

    par.alloc(Loc::new(left.start, right.end, Expression::Member {
        object: left,
        property: right,
    }))
});

const CALL: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let arguments = par.expression_list();

    par.alloc(Expression::Call {
        callee: left,
        arguments,
    }.at(0, 0))
});

const CMEM: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let property = par.expression(0);

    expect!(par, BracketClose);

    par.alloc(Expression::ComputedMember {
        object: left,
        property: property,
    }.at(0, 0))
});

const TPLS: NestedHandler = Some(|par, left| {
    par.template_string(Some(left))
});

const TPLE: NestedHandler = Some(|par, left| {
    par.template_expression(Some(left))
});

macro_rules! binary_handlers {
    ($( [ $( $inner:tt )* ] )+) => {
        $( binary_handlers!( $( $inner )* ); )*
    };

    ($name:ident @ $bp:expr => $op:ident) => {
        const $name: NestedHandler = Some(|par, left| {
            par.lexer.consume();

            let right = par.expression($bp);

            par.alloc(Loc::new(left.start, right.end, Expression::Binary {
                operator: $op,
                left,
                right,
            }))
        });
    };
}

binary_handlers! {
    [ASGN @ 3  => Assign]
    [ADDA @ 3  => AddAssign]
    [SUBA @ 3  => SubtractAssign]
    [EXPA @ 3  => ExponentAssign]
    [MULA @ 3  => MultiplyAssign]
    [DIVA @ 3  => DivideAssign]
    [REMA @ 3  => RemainderAssign]
    [BSLA @ 3  => BSLAssign]
    [BSRA @ 3  => BSRAssign]
    [UBSA @ 3  => UBSRAssign]
    [BWAA @ 3  => BitAndAssign]
    [XORA @ 3  => BitXorAssign]
    [BORA @ 3  => BitOrAssign]
    [OR   @ 5  => LogicalOr]
    [AND  @ 6  => LogicalAnd]
    [BWOR @ 7  => BitwiseOr]
    [BWXO @ 8  => BitwiseXor]
    [BWAN @ 9  => BitwiseAnd]
    [STEQ @ 10 => StrictEquality]
    [SIEQ @ 10 => StrictInequality]
    [EQ   @ 10 => Equality]
    [INEQ @ 10 => Inequality]
    [LESS @ 11 => Lesser]
    [LSEQ @ 11 => LesserEquals]
    [GRTR @ 11 => Greater]
    [GREQ @ 11 => GreaterEquals]
    [INOF @ 11 => Instanceof]
    [IN   @ 11 => In]
    [BSL  @ 12 => BitShiftLeft]
    [BSR  @ 12 => BitShiftRight]
    [UBSR @ 12 => UBitShiftRight]
    [ADD  @ 13 => Addition]
    [SUB  @ 13 => Subtraction]
    [MUL  @ 14 => Multiplication]
    [DIV  @ 14 => Division]
    [REM  @ 14 => Remainder]
    [EXPN @ 15 => Exponent]
}


impl<'ast> Parser<'ast> {
    #[inline]
    pub fn expression(&mut self, bp: u8) -> ExpressionPtr<'ast> {
        let parent = self.bound_expression();

        self.nested_expression(parent, bp)
    }

    #[inline]
    pub fn nested_expression(&mut self, mut parent: ExpressionPtr<'ast>, bp: u8) -> ExpressionPtr<'ast> {
        let table = lookup(bp);

        loop {
            parent = match unsafe { *table.offset(self.lexer.token as isize) } {
                Some(handler) => handler(self, parent),
                None          => return parent,
            }
        }
    }
}
