use std::mem;

use ast::{Program, Store, Statement, Expression};
use error::{Error, ParseResult, Result};
use tokenizer::Tokenizer;
use lexicon::Token;
use lexicon::Token::*;

/// Peek on the next token. Return with an error if tokenizer fails.
macro_rules! peek {
    ($parser:ident) => {
        match $parser.token {
            Some(token) => token,

            None => {
                let token = try!($parser.tokenizer.get_token());

                $parser.token = Some(token);

                token
            }
        }
    }
}

/// Get the next token. Return with an error if tokenizer fails.
macro_rules! next {
    ($parser:ident) => {
        match $parser.token {
            Some(token) => {
                $parser.consume();

                token
            },
            None => try!($parser.tokenizer.get_token())
        }
    }
}

/// If the next token matches `$p`, consume that token and execute `$eval`.
macro_rules! allow {
    ($parser:ident, $p:pat => $eval:expr) => {
        match peek!($parser) {
            $p => {
                $parser.consume();
                $eval;
            },
            _ => {}
        }
    }
}

/// Return an error if the next token doesn't match $p.
macro_rules! expect {
    ($parser:ident, $p:pat) => {
        match next!($parser) {
            $p => {},
            _  => unexpected_token!($parser)
        }
    }
}

/// Expect the next token to be an Identifier, extracting the OwnedSlice
/// out of it. Returns an error otherwise.
macro_rules! expect_identifier {
    ($parser:ident) => {
        match next!($parser) {
            Identifier(ident) => ident,
            _                 => unexpected_token!($parser)
        }
    }
}

/// Expecta semicolon to terminate a statement. Will assume a semicolon
/// following the ASI rules.
macro_rules! expect_semicolon {
    ($parser:ident) => {
        // TODO: Tokenizer needs to flag when a new line character has been
        //       consumed to satisfy all ASI rules
        match peek!($parser) {
            Semicolon     => $parser.consume(),

            ParenClose    |
            BraceClose    |
            EndOfProgram  => {},

            _             => {
                if !$parser.tokenizer.asi() {
                    unexpected_token!($parser)
                }
            }
        }
    }
}

/// Return an error for current token.
macro_rules! unexpected_token {
    ($parser:ident) => {
        return Err($parser.tokenizer.invalid_token())
    };
}

pub struct Parser<'src> {
    // Tokenizer will produce tokens from the source
    tokenizer: Tokenizer<'src>,

    // Current token, to be used by peek! and next! macros
    token: Option<Token>,

    program: Program<'src>,

    // expressions: Store<Expression>,
    // statements: Store<Statement>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source),
            token: None,
            program: Program {
                source: source,
                expressions: Store::new(),
                statements: Store::new(),
            }
            // expressions: Store::new(),
            // statements: Store::new(),
        }
    }

    #[inline]
    fn consume(&mut self) {
        self.token = None;
    }

    #[inline]
    fn statement(&mut self, token: Token) -> Result<Statement> {
        match token {
            // Semicolon          => Ok(Statement::Empty),
            // BraceOpen          => self.block_statement(),
            // Declaration(kind)  => self.variable_declaration_statement(kind),
            // Return             => self.return_statement(),
            // Break              => self.break_statement(),
            // Function           => self.function_statement(),
            // Class              => self.class_statement(),
            // If                 => self.if_statement(),
            // While              => self.while_statement(),
            // Do                 => self.do_statement(),
            // For                => self.for_statement(),
            // Identifier(label)  => self.labeled_or_expression_statement(label),
            // Throw              => self.throw_statement(),
            // Try                => self.try_statement(),
            _                  => self.expression_statement(token),
        }
    }

    #[inline]
    fn expression_from(&mut self, token: Token) -> Result<Expression> {
        Ok(match token {
            Identifier(value)  => Expression::Identifier(value.into()),
            _                  => unexpected_token!(self)
        })
    }

    #[inline]
    fn expression_statement(&mut self, token: Token) -> Result<Statement> {
        let expression = try!(self.expression_from(token));

        let id = self.program.expressions.insert(0, 0, expression);

        expect_semicolon!(self);

        Ok(Statement::Expression(id))
    }

    #[inline]
    pub fn parse(&mut self) -> Result<()> {
        let mut previous = 0usize;

        loop {
            let statement = match next!(self) {
                EndOfProgram => break,
                token        => try!(self.statement(token))
            };

            let id = self.program.statements.insert(0, 0, statement);

            self.program.statements[previous].next = Some(id);

            previous = id;
        }

        Ok(())
    }
}

pub fn parse<'src>(source: &'src str) -> Result<Program<'src>> {
    let mut parser = Parser::new(source);

    parser.parse()?;

    Ok(parser.program)
}

#[cfg(test)]
mod test {
    use super::*;
    use ast::{Ident, Slice};

    #[test]
    fn parse_ident_expr() {
        let src = "foo;bar;baz;";

        let program = parse(src).unwrap();

        let exprs = &program.expressions;
        let stmts = &program.statements;

        // Statements are there and refer to correct expressions
        assert_eq!(stmts.len(), 3);
        assert_eq!(stmts[0].value, Statement::Expression(0));
        assert_eq!(stmts[1].value, Statement::Expression(1));
        assert_eq!(stmts[2].value, Statement::Expression(2));

        // Statements are linked
        assert_eq!(stmts[0].next, Some(1));
        assert_eq!(stmts[1].next, Some(2));
        assert_eq!(stmts[2].next, None);

        // Expressions are there and hold correct slices
        assert_eq!(exprs.len(), 3);

        match exprs[0].value {
            Expression::Identifier(ref ident) => assert_eq!(ident.as_str(src), "foo"),
            _ => panic!()
        }
        match exprs[1].value {
            Expression::Identifier(ref ident) => assert_eq!(ident.as_str(src), "bar"),
            _ => panic!()
        }
        match exprs[2].value {
            Expression::Identifier(ref ident) => assert_eq!(ident.as_str(src), "baz"),
            _ => panic!()
        }
    }
}
