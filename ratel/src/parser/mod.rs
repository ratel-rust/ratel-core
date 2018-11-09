#[macro_use]
mod macros;
mod error;
mod expression;
mod statement;
mod function;
mod nested;
mod feature;

use toolshed::list::ListBuilder;
use toolshed::Arena;
use error::Error;
use module::Module;

use self::error::ToError;
use self::nested::*;
use self::feature::{FeatureSet, ES5, ES2015};

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

    set: &'static FeatureSet,
}

lazy_static! {
    static ref DEFAULT_FEATURE_SET: FeatureSet = {
        let mut set = FeatureSet::default();

        ES5(&mut set);
        ES2015(&mut set);

        set
    };
}

impl<'ast> Parser<'ast> {
    pub fn new(source: &str, arena: &'ast Arena) -> Self {
        Parser {
            arena,
            lexer: Lexer::new(arena, source),
            errors: Vec::new(),
            body: NodeList::empty(),
            set: &DEFAULT_FEATURE_SET,
        }
    }

    fn error<T: ToError>(&mut self) -> T {
        let err = self.lexer.invalid_token();

        self.errors.push(err);

        T::to_error()
    }

    fn asi(&mut self) -> Asi {
        self.lexer.asi()
    }

    /// Create a `Node<'ast, T>` at a specified location.
    fn node_at<T, I>(&mut self, start: u32, end: u32, item: I) -> Node<'ast, T> where
        T: Copy,
        I: Into<T>,
    {
        Node::new(self.arena.alloc(Loc::new(start, end, item.into())))
    }

    /// Create a `Node<'ast, T>` at current token location, without consuming the token.
    fn node<T, I>(&mut self, item: I) -> Node<'ast, T> where
        T: Copy,
        I: Into<T>,
    {
        let (start, end) = self.lexer.loc();

        self.node_at(start, end, item)
    }

    /// Create a `Node<'ast, T>` at current token location, consuming the token.
    fn node_consume<T, I>(&mut self, item: I) -> Node<'ast, T> where
        T: Copy,
        I: Into<T>,
    {
        let item = self.node(item);
        self.lexer.consume();
        item
    }

    /// Create a `Node<'ast, T>` at current token location from its `&str` value,
    /// consuming the token.
    fn node_consume_str<T, F, I>(&mut self, make_item: F) -> Node<'ast, T> where
        T: Copy,
        F: FnOnce(&'ast str) -> I,
        I: Into<T>,
    {
        let value = self.lexer.token_as_str();

        self.node_consume(make_item(value))
    }

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

        self.node_at(start, end, block)
    }

    /// Same as above, but assumes that the opening brace has already been checked
    fn unchecked_block<I>(&mut self) -> BlockNode<'ast, I> where
        I: Parse<'ast, Output = Node<'ast, I>> + Copy
    {
        let start = self.lexer.start_then_consume();
        let block = self.raw_block();
        let end   = self.lexer.end_then_consume();

        self.node_at(start, end, block)
    }

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

    fn identifier(&mut self) -> IdentifierNode<'ast> {
        match self.lexer.token {
            Identifier => self.node_consume_str(|ident| ident),
            _          => self.error()
        }
    }

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

        self.node_at(expression.start, expression.end, pattern)
    }

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

        pub fn block<'a, I, T, L>(&'a self, list: L) -> BlockNode<'a, I> where
            I: Copy,
            T: Into<I> + Copy,
            L: AsRef<[T]>
        {
            self.ptr(Block { body: self.list(list) })
        }

        pub fn empty_block<'a, I: Copy>(&'a self) -> BlockNode<'a, I> {
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
