#[macro_use]
mod macros;
mod error;
mod expression;
mod statement;
mod function;
mod nested;

use toolshed::list::ListBuilder;
use toolshed::Arena;
use error::Error;
use module::Module;

use self::error::ToError;
use self::nested::*;

use ast::{Loc, Node, Statement, NodeList, Block, BlockNode};
use ast::{Expression, ExpressionNode, ExpressionList, IdentifierNode};
use ast::{OperatorKind, Pattern};
use ast::expression::BinaryExpression;
use lexer::{Lexer, Asi};
use lexer::Token::*;

pub trait Parse<'ast> {
    type Output;

    fn parse(&mut Parser<'ast>) -> Self::Output;
}

pub struct Parser<'ast> {
    arena: &'ast Arena,

    /// Lexer will produce tokens from the source
    lexer: Lexer<'ast>,

    /// Errors occurred during parsing
    errors: Vec<Error>,

    /// AST under construction
    body: NodeList<'ast, Statement<'ast>>,
}

impl<'ast> Parser<'ast> {
    pub fn new(source: &str, arena: &'ast Arena) -> Self {
        Parser {
            arena,
            lexer: Lexer::new(arena, source),
            errors: Vec::new(),
            body: NodeList::empty(),
        }
    }

    fn error<T: ToError>(&mut self) -> T {
        let err = self.lexer.invalid_token();

        self.errors.push(err);

        T::to_error()
    }

    #[inline]
    fn asi(&mut self) -> Asi {
        self.lexer.asi()
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
    fn alloc<T>(&mut self, val: Loc<T>) -> Node<'ast, T> where
        T: Copy,
    {
        Node::new(self.arena.alloc(val))
    }

    #[inline]
    fn alloc_in_loc<T, I>(&mut self, item: I) -> Node<'ast, T> where
        T: Copy,
        I: Into<T>,
    {
        let node = self.in_loc(item.into());
        self.alloc(node)
    }

    #[inline]
    fn alloc_at_loc<T, I>(&mut self, start: u32, end: u32, item: I) -> Node<'ast, T> where
        T: Copy,
        I: Into<T>,
    {
        self.alloc(Loc::new(start, end, item.into()))
    }

    #[inline]
    fn parse(&mut self) {
        if self.lexer.token == EndOfProgram {
            return;
        }

        let statement = self.statement();
        let builder = ListBuilder::new(self.arena, statement);

        while self.lexer.token != EndOfProgram {
            builder.push(self.arena, self.statement());
        }

        self.body = builder.as_list()
    }

    #[inline]
    fn block<I>(&mut self) -> BlockNode<'ast, I> where
        I: Parse<'ast, Output = Node<'ast, I>> + Copy
    {
        let start = self.lexer.start();

        match self.lexer.token {
            BraceOpen => self.lexer.consume(),
            _         => self.error::<()>(),
        }

        let block = self.raw_block();
        let end   = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, block)
    }

    /// Same as above, but assumes that the opening brace has already been checked
    #[inline]
    fn unchecked_block<I>(&mut self) -> BlockNode<'ast, I> where
        I: Parse<'ast, Output = Node<'ast, I>> + Copy
    {
        let start = self.lexer.start_then_consume();
        let block = self.raw_block();
        let end   = self.lexer.end_then_consume();

        self.alloc_at_loc(start, end, block)
    }

    #[inline]
    fn raw_block<I>(&mut self) -> Block<'ast, I> where
        I: Parse<'ast, Output = Node<'ast, I>> + Copy
    {
        if self.lexer.token == BraceClose {
            return Block { body: NodeList::empty() };
        }

        let statement = I::parse(self);
        let builder = ListBuilder::new(self.arena, statement);

        while self.lexer.token != BraceClose && self.lexer.token != EndOfProgram {
            builder.push(self.arena, I::parse(self));
        }

        Block { body: builder.as_list() }
    }

    #[inline]
    fn identifier(&mut self) -> IdentifierNode<'ast> {
        match self.lexer.token {
            Identifier => {
                let ident = self.lexer.token_as_str();
                let ident = self.alloc_in_loc(ident);
                self.lexer.consume();
                ident
            },
            _ => self.error()
        }
    }

    #[inline]
    fn pattern_from_expression(&mut self, expression: ExpressionNode<'ast>) -> Node<'ast, Pattern<'ast>> {
        let pattern = match expression.item {
            Expression::Binary(BinaryExpression {
                operator: OperatorKind::Assign,
                left,
                right,
            }) => {
                Pattern::AssignmentPattern {
                    left: self.pattern_from_expression(left),
                    right
                }
            },
            Expression::Identifier(ident) => {
                Pattern::Identifier(ident)
            },
            _ => self.error()
        };

        self.alloc_at_loc(expression.start, expression.end, pattern)
    }

    #[inline]
    fn params_from_expressions(&mut self, expressions: ExpressionList<'ast>) -> NodeList<'ast, Pattern<'ast>> {
        let mut expressions = expressions.iter();

        let builder = match expressions.next() {
            Some(&expression) => {
                let param = self.pattern_from_expression(expression);

                ListBuilder::new(self.arena, param)
            },
            None => return NodeList::empty()
        };

        for &expression in expressions {
            builder.push(self.arena, self.pattern_from_expression(expression));
        }

        builder.as_list()
    }
}

/// Parse the JavaScript source `&str` and produce an Abstract Syntax Tree `Module`.
pub fn parse<'src, 'ast>(source: &'src str) -> Result<Module<'ast>, Vec<Error>> {
    let arena = Arena::new();

    let (body, errors) = {
        let mut parser = Parser::new(source, &arena);

        parser.parse();

        (parser.body.into_unsafe(), parser.errors)
    };

    match errors.len() {
        0 => Ok(Module::new(body, arena)),
        _ => Err(errors)
    }
}

#[cfg(test)]
mod mock {
    use super::*;
    use ast::{Literal, ExpressionNode, Block, BlockNode, Name};

    pub struct Mock {
        arena: Arena
    }

    impl Mock {
        pub fn new() -> Self {
            Mock {
                arena: Arena::new()
            }
        }

        pub fn ptr<'a, T, I>(&'a self, val: I) -> Node<'a, T> where
            T: 'a + Copy,
            I: Into<T>,
        {
            Node::new(self.arena.alloc(Loc::new(0, 0, val.into())))
        }

        pub fn name<'a, N>(&'a self, val: &'a str) -> N where
            N: Name<'a> + From<Node<'a, &'a str>>,
        {
            N::from(Node::new(self.arena.alloc(Loc::new(0, 0, val))))
        }

        pub fn number<'a>(&'a self, number: &'static str) -> ExpressionNode<'a> {
            self.ptr(Literal::Number(number))
        }

        pub fn block<I, T, L>(&self, list: L) -> BlockNode<I> where
            I: Copy,
            T: Into<I> + Copy,
            L: AsRef<[T]>
        {
            self.ptr(Block { body: self.list(list) })
        }

        pub fn empty_block<I: Copy>(&self) -> BlockNode<I> {
            self.ptr(Block { body: NodeList::empty() })
        }

        pub fn list<'a, T, I, L>(&'a self, list: L) -> NodeList<'a, T> where
            T: 'a + Copy,
            L: AsRef<[I]>,
            I: Into<T> + Copy,
        {
            NodeList::from_iter(&self.arena, list.as_ref().iter().cloned().map(|i| {
                Node::new(self.arena.alloc(Loc::new(0, 0, i.into())))
            }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::mock::Mock;

    #[test]
    fn empty_parse() {
        assert_eq!(parse("").unwrap().body(), NodeList::empty());
    }

    #[test]
    fn empty_statements() {
        let mock = Mock::new();

        let expected = mock.list([
            Statement::Empty,
            Statement::Empty,
            Statement::Empty
        ]);

        assert_eq!(parse(";;;").unwrap().body(), expected);
    }
}
