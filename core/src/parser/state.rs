use parser::Parser;
use lexer::Token::*;
use lexer::{Asi, Token};
use ast::{Ptr, Loc, List, ListBuilder, Declarator, DeclarationKind};
use ast::{Statement, StatementPtr, Expression, ExpressionPtr, Value};
use ast::OperatorKind;
use ast::OperatorKind::*;
use ast::{EmptyListBuilder};


type StatementHandler = for<'ast> fn(&mut Parser<'ast>) -> StatementPtr<'ast>;

static STMT_HANDLERS: [StatementHandler; 108] = [
    ____, EMPT, ____, ____, PRN,  ____, ARR,  ____, BLCK, ____, ____, OP,
//  EOF   ;     :     ,     (     )     [     ]     {     }     =>    NEW

    OP,   OP,   OP,   OP,   OP,   OP,   OP,   ____, REG,  ____, ____, OP,
//  ++    --    !     ~     TYPOF VOID  DELET *     /     %     **    +

    OP,   ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  -     <<    >>    >>>   <     <=    >     >=    INSOF IN    ===   !==

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, ____,
//  ==    !=    &     ^     |     &&    ||    ?     =     +=    -=    **=

    ____, ____, ____, ____, ____, ____, ____, ____, ____, ____, VAR,  LET,
//  *=    /=    %=    <<=   >>=   >>>=  &=    ^=    |=    ...   VAR   LET

    CONS, BRK,  DO,   ____, ____, ____, ____, CLAS, ____, RET,  WHL,  ____,
//  CONST BREAK DO    CASE  ELSE  CATCH EXPRT CLASS EXTND RET   WHILE FINLY

    ____, ____, CONT, FOR,  SWCH, ____, ____, FUNC, THIS, ____, IF,   THRW,
//  SUPER WITH  CONT  FOR   SWTCH YIELD DBGGR FUNCT THIS  DEFLT IF    THROW

    ____, TRY,  ____, TRUE, FALS, NULL, UNDE, STR,  NUM,  BIN,  ____, ____,
//  IMPRT TRY   STATI TRUE  FALSE NULL  UNDEF STR   NUM   BIN   REGEX ENUM

    ____, ____, ____, ____, ____, ____, IDEN, ____, TPL,  TPL,  ____, ____,
//  IMPL  PCKG  PROT  IFACE PRIV  PUBLI IDENT ACCSS TPL_O TPL_C ERR_T ERR_E
];

const ____: StatementHandler = |par| unexpected_token!(par);

const EMPT : StatementHandler = |par| {
    let stmt = par.alloc_in_loc(Statement::Empty);
    par.lexer.consume();

    stmt
};

const PRN : StatementHandler = |par| {
    par.lexer.consume();
    let expr = par.paren_expression();

    par.expression_statement(expr)
};

const ARR : StatementHandler = |par| {
    par.lexer.consume();
    let expr = par.array_expression();

    par.expression_statement(expr)
};

const BLCK : StatementHandler = |par| {
    par.lexer.consume();
    par.block_statement()
};

const OP  : StatementHandler = |par| {
    let op = OperatorKind::from_token(par.lexer.token).expect("Must be a prefix operator");
    par.lexer.consume();
    let expr = par.prefix_expression(op);

    par.expression_statement(expr)
};

const REG : StatementHandler = |par| {
    let expr = par.regular_expression();

    par.expression_statement(expr)
};

const VAR: StatementHandler = |par| {
    par.lexer.consume();
    par.variable_declaration_statement(DeclarationKind::Var)
};

const LET: StatementHandler = |par| {
    par.lexer.consume();
    par.variable_declaration_statement(DeclarationKind::Let)
};

const CONS: StatementHandler = |par| {
    par.lexer.consume();
    par.variable_declaration_statement(DeclarationKind::Const)
};

const RET: StatementHandler = |par| {
    par.lexer.consume();
    par.return_statement()
};

const BRK: StatementHandler = |par| {
    par.lexer.consume();
    par.break_statement()
};

const CONT: StatementHandler = |par| {
    par.lexer.consume();
    par.continue_statement()
};

const CLAS: StatementHandler = |par| {
    par.lexer.consume();
    par.class_statement()
};

const IF: StatementHandler = |par| {
    par.lexer.consume();
    par.if_statement()
};

const WHL: StatementHandler = |par| {
    par.lexer.consume();
    par.while_statement()
};

const DO: StatementHandler = |par| {
    par.lexer.consume();
    par.do_statement()
};

const FOR: StatementHandler = |par| {
    par.lexer.consume();
    par.for_statement()
};

const THRW: StatementHandler = |par| {
    par.lexer.consume();
    par.throw_statement()
};

const TRY: StatementHandler = |par| {
    par.lexer.consume();
    par.try_statement()
};

const SWCH: StatementHandler = |par| {
    par.lexer.consume();
    par.switch_statement()
};

const FUNC: StatementHandler = |par| {
    par.lexer.consume();
    par.function_statement()
};

const THIS: StatementHandler = |par| {
    let expr = par.alloc_in_loc(Expression::This);
    par.lexer.consume();

    par.expression_statement(expr)
};

const TRUE: StatementHandler = |par| {
    let expr = par.alloc_in_loc(Expression::Value(Value::True));
    par.lexer.consume();

    par.expression_statement(expr)
};

const FALS: StatementHandler = |par| {
    let expr = par.alloc_in_loc(Expression::Value(Value::False));

    par.lexer.consume();
    par.expression_statement(expr)
};

const NULL: StatementHandler = |par| {
    let expr = par.alloc_in_loc(Expression::Value(Value::Null));

    par.lexer.consume();
    par.expression_statement(expr)
};

const UNDE: StatementHandler = |par| {
    let expr = par.alloc_in_loc(Expression::Value(Value::Undefined));

    par.lexer.consume();
    par.expression_statement(expr)
};

const STR : StatementHandler = |par| {
    let value = par.lexer.token_as_str();
    let expr = par.alloc_in_loc(Expression::Value(Value::String(value)));

    par.lexer.consume();
    par.expression_statement(expr)
};

const NUM : StatementHandler = |par| {
    let value = par.lexer.token_as_str();
    let expr = par.alloc_in_loc(Expression::Value(Value::Number(value)));

    par.lexer.consume();
    par.expression_statement(expr)
};

const BIN : StatementHandler = |par| {
    let value = par.lexer.token_as_str();
    let expr = par.alloc_in_loc(Expression::Value(Value::Binary(value)));

    par.lexer.consume();
    par.expression_statement(expr)
};

const IDEN: StatementHandler = |par| {
    let label = par.lexer.token_as_str();
    par.lexer.consume();
    par.labeled_or_expression_statement(label)
};

const TPL : StatementHandler = |par| {
    let expr = par.template_expression(None);
    par.expression_statement(expr)
};

enum State {
    Body {
        lookup: *const Option<for<'ast> fn(&'ast Parser) -> StatementPtr<'ast>>,
        builder: ListBuilder<'ast, Statement<'ast>>,
    },
    NestedExpression {
        lookup: *const Option<for<'ast> fn(&'ast Parser, ExpressionPtr<'ast>, u8) -> ExpressionPtr<'ast>>,
        parent: ExpressionPtr<'ast>,
        binding_power: u8
    }
}

impl<'ast> Parser<'ast> {
    #[inline]
    fn run(&mut self) {
        let mut stack = Vec::new();

        loop {
            match *stack.last_mut() {
                None => {
                    break;
                }

                Some(ref mut state) => {
                    *state = match state {
                        State::Body { lookup, ref mut builder } => {

                        },
                        State::NestedExpression { lookup, ref mut parent, binding_power } => {
                            let handler = unsafe { *lookup.offset(self.lexer.token as isize) };

                            match handler {
                                Some(handler) => *parent = handler(self, *parent, binding_power),
                                None          =>
                            }
                        }
                    }
                }
            }
        }


        if self.lexer.token == EndOfProgram {
            return;
        }

        let statement = self.statement();
        let mut builder = ListBuilder::new(self.arena, statement);

        while self.lexer.token != EndOfProgram {
            builder.push(self.statement());
        }

        self.body = builder.into_list()
    }
}
