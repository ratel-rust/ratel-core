#[macro_use]
mod macros;
mod error;
mod expression;
mod statement;
mod function;

use error::Error;
use arena::Arena;
use module::Module;

use ast::{Loc, Ptr, Statement, List, ListBuilder, EmptyListBuilder};
use ast::{Parameter, ParameterKey, ParameterPtr, ParameterList, OperatorKind};
use ast::{Expression, ExpressionPtr, ExpressionList};
use lexer::{Lexer, Token, Asi};
use lexer::Token::*;

pub struct Parser<'ast> {
    arena: &'ast Arena,

    /// Lexer will produce tokens from the source
    lexer: Lexer<'ast>,

    /// Set to `Some` whenever peek is called
    token: Option<Token<'ast>>,

    /// Errors occurred during parsing
    errors: Vec<Error>,

    /// AST under construction
    body: List<'ast, Loc<Statement<'ast>>>,
}

impl<'ast> Parser<'ast> {
    pub fn new(source: &'ast str, arena: &'ast Arena) -> Self {
        Parser {
            arena,
            lexer: Lexer::new(source),
            token: None,
            errors: Vec::new(),
            body: List::empty(),
        }
    }

    /// Get the next token.
    #[inline]
    fn next(&mut self) -> Token<'ast> {
        match self.token {
            None => self.lexer.get_token(),

            Some(token) => {
                self.token = None;

                token
            }
        }
    }

    /// Peek on the next token.
    #[inline]
    fn peek(&mut self) -> Token<'ast> {
        match self.token {
            None => {
                let token = self.lexer.get_token();

                self.token = Some(token);

                token
            },

            Some(token) => token
        }
    }

    #[inline]
    fn asi(&mut self) -> Asi {
        self.peek();

        self.lexer.asi()
    }

    #[inline]
    fn consume(&mut self) {
        self.token = None;
    }

    #[inline]
    fn loc(&self) -> (u32, u32) {
        self.lexer.loc()
    }

    #[inline]
    fn in_loc<T>(&self, item: T) -> Loc<T> {
        let (start, end) = self.loc();

        Loc::new(start, end, item)
    }

    #[inline]
    fn alloc<T: Copy>(&mut self, val: T) -> Ptr<'ast, T> {
        Ptr::new(self.arena.alloc(val))
    }

    #[inline]
    fn alloc_in_loc<T: Copy>(&mut self, item: T) -> Ptr<'ast, Loc<T>> {
        let node = self.in_loc(item);
        self.alloc(node)
    }

    #[inline]
    fn parse(&mut self) {
        let statement = match self.next() {
            EndOfProgram => return,
            token        => self.statement(token)
        };

        let mut builder = ListBuilder::new(self.arena, statement);

        loop {
            let statement = match self.next() {
                EndOfProgram => break,
                token        => self.statement(token)
            };

            builder.push(statement);
        }

        self.body = builder.into_list()
    }

    #[inline]
    fn block_body_tail(&mut self) -> List<'ast, Loc<Statement<'ast>>> {
        let statement = match self.next() {
            BraceClose => return List::empty(),
            token      => self.statement(token),
        };

        let mut builder = ListBuilder::new(self.arena, statement);

        loop {
            let statement = match self.next() {
                BraceClose => break,
                token      => self.statement(token),
            };

            builder.push(statement);
        }

        builder.into_list()
    }

    #[inline]
    fn block_body(&mut self) -> List<'ast, Loc<Statement<'ast>>> {
        expect!(self, BraceOpen);
        self.block_body_tail()
    }

    #[inline]
    fn param_from_expression(&mut self, expression: ExpressionPtr<'ast>) -> ParameterPtr<'ast> {
        match expression.item {
            Expression::Identifier(ident) => {
                let param = Parameter {
                    key: ParameterKey::Identifier(ident),
                    value: None
                };

                self.alloc_in_loc(param)
            },
            Expression::Binary {
                operator: OperatorKind::Assign,
                left,
                right
            } => {
                let param = Parameter {
                    key: self.param_from_expression(left).key,
                    value: Some(right)
                };

                self.alloc_in_loc(param)
            },
            _ => panic!("Unexpected token")
        }
    }

    #[inline]
    fn params_from_expressions(&mut self, expressions: ExpressionList<'ast>) -> ParameterList<'ast> {
        let mut expressions = expressions.ptr_iter();

        let mut builder = match expressions.next() {
            Some(expression) => {
                if let Expression::Sequence { body } = expression.item {
                    return self.params_from_expressions(body);
                }

                let param = self.param_from_expression(*expression);
                ListBuilder::new(self.arena, param)
            },
            None => return List::empty()
        };

        for expression in expressions {
            builder.push(self.param_from_expression(*expression));
        }

        builder.into_list()
    }

    #[inline]
    fn parameter_list(&mut self) -> ParameterList<'ast> {
        let mut builder = EmptyListBuilder::new(self.arena);

        loop {
            let key = parameter_key!(self);
            let token = self.next();

            if token == Operator(OperatorKind::Assign) {
                return self.parameter_list_with_defaults(key, builder);
            }

            let parameter = Parameter {
                key,
                value: None
            };

            builder.push(self.alloc_in_loc(parameter));

            match token {
                ParenClose => break,
                Comma      => {},
                _          => unexpected_token!(self),
            };
        }

        builder.into_list()
    }

    #[inline]
    fn parameter_list_with_defaults(
        &mut self,
        mut key: ParameterKey<'ast>,
        mut builder: EmptyListBuilder<'ast, Loc<Parameter<'ast>>>
    ) -> ParameterList<'ast> {
        loop {
            let value = self.expression(0);
            let parameter = Parameter {
                key,
                value: Some(value)
            };

            builder.push(self.alloc_in_loc(parameter));

            match self.next() {
                ParenClose => break,
                Comma      => {},
                _          => unexpected_token!(self),
            };

            key = parameter_key!(self);

            expect!(self, Operator(OperatorKind::Assign));
        }

        builder.into_list()
    }
}

pub fn parse(source: &str) -> Result<Module, Vec<Error>> {
    let arena = Arena::new();

    let (body, errors) = {
        let source = arena.alloc_str(source);
        let mut parser = Parser::new(source, &arena);

        parser.parse();

        (parser.body.into_raw(), parser.errors)
    };

    match errors.len() {
        0 => Ok(Module::new(body, arena)),
        _ => Err(errors)
    }
}

#[cfg(test)]
mod mock {
    use super::*;
    use ast::{Expression, Value};

    pub struct Mock {
        arena: Arena
    }

    impl Mock {
        pub fn new() -> Self {
            Mock {
                arena: Arena::new()
            }
        }

        pub fn ptr<'a, T: 'a + Copy>(&'a self, val: T) -> Ptr<'a, Loc<T>> {
            Ptr::new(self.arena.alloc(Loc::new(0, 0, val)))
        }

        pub fn ident<'a>(&'a self, ident: &'static str) -> Ptr<'a, Loc<Expression<'a>>> {
            self.ptr(Expression::Identifier(ident))
        }

        pub fn number<'a>(&'a self, number: &'static str) -> Ptr<'a, Loc<Expression<'a>>> {
            self.ptr(Expression::Value(Value::Number(number)))
        }

        pub fn list<'a, T, L>(&'a self, list: L) -> List<'a, Loc<T>> where
            T: 'a + Copy,
            L: AsRef<[T]>
        {
            List::from_iter(&self.arena, list.as_ref().iter().cloned().map(|i| Loc::new(0, 0, i)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::mock::Mock;

    #[test]
    fn empty_parse() {
        let module = parse("").unwrap();

        assert_eq!(module.body(), List::empty());
    }

    #[test]
    fn empty_statements() {
        let module = parse(";;;").unwrap();
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Empty,
            Statement::Empty,
            Statement::Empty
        ]);

        assert_eq!(module.body(), expected);
    }

}
