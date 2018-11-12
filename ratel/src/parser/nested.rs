use toolshed::list::ListBuilder;
use parser::Parser;
use lexer::Token;
use lexer::Token::*;
use ast::{NodeList, Expression, ExpressionNode};
use ast::expression::*;
use ast::OperatorKind::*;
use lexer::Asi;

const TOTAL_TOKENS: usize = 108;

type NestedHandler = Option<for<'ast> fn(&mut Parser<'ast>, ExpressionNode<'ast>) -> ExpressionNode<'ast>>;

pub trait BindingPower {
    const LUT: [NestedHandler; TOTAL_TOKENS];

    #[inline]
    fn handler(asi: Asi, token: Token) -> NestedHandler {
        // TODO: find a cleaner solution, roll it the ASI check into lookup table somehow?
        if asi == Asi::ImplicitSemicolon {
            match token {
                OperatorIncrement | OperatorDecrement => return None,
                _ => {}
            }
        }

        Self::LUT[token as usize]
    }
}

macro_rules! bp {
    ($name:ident, $table:tt) => {
        pub struct $name;

        impl BindingPower for $name {
            const LUT: [NestedHandler; TOTAL_TOKENS] = $table;
        }
    }
}

/// All potential tokens, including Comma for sequence expressions
bp!(ANY, [
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
]);

bp!(B0, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  OR,   COND, ASGN, ADDA, SUBA, EXPA,
    MULA, DIVA, REMA, BSLA, BSRA, UBSA, BWAA, XORA, BORA, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B1, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  OR,   COND, ASGN, ADDA, SUBA, EXPA,
    MULA, DIVA, REMA, BSLA, BSRA, UBSA, BWAA, XORA, BORA, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B5, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, AND,  ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B6, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, BWOR, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B7, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, BWXO, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B8, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, BWAN, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B9, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   STEQ, SIEQ,
    EQ,   INEQ, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B10, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, LESS, LSEQ, GRTR, GREQ, INOF, IN,   ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B11, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  BSL,  BSR,  UBSR, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B12, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ADD,
    SUB,  ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B13, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, MUL,  DIV,  REM,  EXPN, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B14, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, ____, ____, ____, EXPN, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

bp!(B15, [
    ____, ____, ____, ____, CALL, ____, CMEM, ____, ____, ____, ARRW, ____,
    INC,  DEC,  ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
    ____, ____, ____, ____, ____, ____, ____, ACCS, TPLE, TPLS, ____, ____,
]);

const ____: NestedHandler = None;

const SEQ: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let builder = ListBuilder::new(par.arena, left);
    builder.push(par.arena, par.expression::<B0>());

    while let Comma = par.lexer.token {
        par.lexer.consume();
        builder.push(par.arena, par.expression::<B0>());
    }
    let end = par.lexer.end();
    par.alloc_at_loc(left.start, end, SequenceExpression {
        body: builder.as_list()
    })
});


const COND: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let consequent = par.expression::<B1>();
    expect!(par, Colon);
    let alternate = par.expression::<B1>();

    par.alloc_at_loc(left.start, alternate.end, ConditionalExpression {
        test: left,
        consequent,
        alternate,
    })
});

const ARRW: NestedHandler = Some(|par, left| {
    par.lexer.consume();

    let params = match left.item {
        Expression::Sequence(SequenceExpression { body }) => body,
        _ => NodeList::from(par.arena, left)
    };

    let expression = par.arrow_function_expression(params);
    let start = left.start;
    let end = par.lexer.end();
    par.alloc_at_loc(start, end, expression)
});

const ACCS: NestedHandler = Some(|par, left| {
    let member = par.lexer.accessor_as_str();
    par.lexer.consume();

    let right = par.alloc_in_loc(member);

    par.alloc_at_loc(left.start, right.end, MemberExpression {
        object: left,
        property: right,
    })
});

const CALL: NestedHandler = Some(|par, left| {
    let start = par.lexer.start_then_consume();
    let arguments = par.call_arguments();
    let end = par.lexer.end_then_consume();

    par.alloc_at_loc(start, end, CallExpression {
        callee: left,
        arguments,
    })
});

const CMEM: NestedHandler = Some(|par, left| {
    par.lexer.consume();
    let property = par.expression::<ANY>();

    expect!(par, BracketClose);
    let end = par.lexer.end();

    par.alloc_at_loc(left.start, end, ComputedMemberExpression {
        object: left,
        property,
    })
});

const TPLS: NestedHandler = Some(|par, left| {
    let quasi = par.template_string();

    par.alloc_at_loc(left.start, quasi.end, TaggedTemplateExpression {
        tag: left,
        quasi,
    })
});

const TPLE: NestedHandler = Some(|par, left| {
    par.tagged_template_expression(left)
});

macro_rules! postfix {
    ($name:ident => $op:ident) => {
        const $name: NestedHandler = {
            fn handler<'ast>(par: &mut Parser<'ast>, left: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
                let end = par.lexer.end();
                par.lexer.consume();

                if !left.is_lvalue() {
                    par.error::<()>();
                }

                par.alloc_at_loc(left.start, end, PostfixExpression {
                    operator: $op,
                    operand: left,
                })
            }

            Some(handler)
        };
    }
}

macro_rules! assign {
    ($name:ident => $op:ident) => {
        const $name: NestedHandler = {
            fn handler<'ast>(par: &mut Parser<'ast>, left: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
                par.lexer.consume();

                if !left.is_lvalue() {
                    par.error::<()>();
                }

                let right = par.expression::<B1>();

                par.alloc_at_loc(left.start, right.end, BinaryExpression {
                    operator: $op,
                    left,
                    right,
                })
            }

            Some(handler)
        };
    }
}

macro_rules! binary {
    ($name:ident, $bp:ident => $op:ident) => {
        const $name: NestedHandler = {
            fn handler<'ast>(par: &mut Parser<'ast>, left: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
                par.lexer.consume();

                let right = par.expression::<$bp>();

                par.alloc_at_loc(left.start, right.end, BinaryExpression {
                    operator: $op,
                    left,
                    right,
                })
            }

            Some(handler)
        };
    }
}

postfix!(INC => Increment);
postfix!(DEC => Decrement);

assign!(ASGN => Assign);
assign!(ADDA => AddAssign);
assign!(SUBA => SubtractAssign);
assign!(EXPA => ExponentAssign);
assign!(MULA => MultiplyAssign);
assign!(DIVA => DivideAssign);
assign!(REMA => RemainderAssign);
assign!(BSLA => BSLAssign);
assign!(BSRA => BSRAssign);
assign!(UBSA => UBSRAssign);
assign!(BWAA => BitAndAssign);
assign!(XORA => BitXorAssign);
assign!(BORA => BitOrAssign);

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
    pub fn nested_expression<B>(&mut self, mut left: ExpressionNode<'ast>) -> ExpressionNode<'ast>
    where
        B: BindingPower
    {
        while let Some(handler) = B::handler(self.asi(), self.lexer.token) {
            left = handler(self, left);
        }

        left
    }
}
