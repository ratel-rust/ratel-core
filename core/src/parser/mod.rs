#[macro_use]
mod macros;
mod error;
mod expression;
mod statement;
mod function;

use error::Error;
use arena::Arena;

use ast::{Loc, Ptr, Statement, RawList, List, ListBuilder, Parameter, ParameterList};
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
    fn alloc<T>(&mut self, val: T) -> Ptr<'ast, T> {
        Ptr::new(self.arena.alloc(val))
    }

    #[inline]
    fn alloc_in_loc<T>(&mut self, item: T) -> Ptr<'ast, Loc<T>> {
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
    fn parameter_list(&mut self) -> ParameterList<'ast> {
        let first = match self.next() {
            Identifier(label) => {
                self.in_loc(Parameter::Identifier {
                    label,
                    value: None
                })
            },
            ParenClose       => return List::empty(),
            _                => unexpected_token!(self),
        };

        let mut builder = ListBuilder::new(self.arena, first);

        loop {
            match self.next() {
                ParenClose => break,
                Comma      => {},
                _          => unexpected_token!(self),
            };

            match self.next() {
                ParenClose        => break,
                Identifier(label) => {
                    let parameter = self.in_loc(Parameter::Identifier {
                        label,
                        value: None
                    });

                    builder.push(parameter);
                },
                _ => unexpected_token!(self)
            }
        }

        builder.into_list()
    }
}

pub struct Module {
    body: RawList,
    arena: Arena,
}

impl Module {
    pub fn body<'ast>(&'ast self) -> List<'ast, Loc<Statement<'ast>>> {
        unsafe { self.body.into_list() }
    }

    pub fn arena(&self) -> &Arena {
        &self.arena
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
        0 => Ok(Module { body, arena }),
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

        pub fn ptr<'a, T: 'a>(&'a self, val: T) -> Ptr<'a, Loc<T>> {
            Ptr::new(self.arena.alloc(Loc::new(0, 0, val)))
        }

        pub fn ident<'a>(&'a self, ident: &'static str) -> Ptr<'a, Loc<Expression<'a>>> {
            self.ptr(Expression::Identifier(ident))
        }

        pub fn number<'a>(&'a self, number: &'static str) -> Ptr<'a, Loc<Expression<'a>>> {
            self.ptr(Expression::Value(Value::Number(number)))
        }

        pub fn list<'a, T, L>(&'a self, list: L) -> List<'a, Loc<T>> where
            T: 'a + Clone,
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
