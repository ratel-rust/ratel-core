// #[macro_export]
// macro_rules! next_raw_ident {
//     ($parser:ident) => ({
//         use ast::OperatorKind::*;
//         use ast::Value::*;

//         match $parser.next() {
//             Identifier(ident)    => ident,
//             Break                => "break",
//             Do                   => "do",
//             Case                 => "case",
//             Else                 => "else",
//             Catch                => "catch",
//             Export               => "export",
//             Class                => "class",
//             Extends              => "extends",
//             Return               => "return",
//             While                => "while",
//             Finally              => "finally",
//             Super                => "super",
//             With                 => "with",
//             Continue             => "continue",
//             For                  => "for",
//             Switch               => "switch",
//             Yield                => "yield",
//             Debugger             => "debugger",
//             Function             => "function",
//             This                 => "this",
//             Default              => "default",
//             If                   => "if",
//             Throw                => "throw",
//             Import               => "import",
//             Try                  => "try",
//             Static               => "static",
//             Operator(New)        => "new",
//             Operator(Typeof)     => "typeof",
//             Operator(Void)       => "void",
//             Operator(Delete)     => "delete",
//             Operator(Instanceof) => "instanceof",
//             Literal(True)        => "true",
//             Literal(False)       => "false",
//             Literal(Null)        => "null",
//             Literal(Undefined)   => "undefined",

//             _                    => unexpected_token!($parser),
//         }
//     })
// }

/// If the next token matches `$p`, consume that token and execute `$eval`.
#[macro_export]
macro_rules! allow {
    ($parser:ident, $p:pat => $eval:expr) => {
        match $parser.peek() {
            $p => {
                $parser.consume();
                $eval;
            },
            _ => {}
        }
    }
}

/// Return an error if the next token doesn't match $p.
#[macro_export]
macro_rules! expect {
    ($parser:ident, $p:pat) => {
        match $parser.next() {
            $p => {},
            _  => unexpected_token!($parser)
        }
    }
}

/// Expect the next token to be an Identifier, extracting the OwnedSlice
/// out of it. Returns an error otherwise.
#[macro_export]
macro_rules! expect_identifier {
    ($parser:ident) => {
        match $parser.next() {
            Token::Identifier(ident) => ident,
            _                        => unexpected_token!($parser)
        }
    }
}

/// Expecta semicolon to terminate a statement. Will assume a semicolon
/// following the ASI rules.
#[macro_export]
macro_rules! expect_semicolon {
    ($parser:ident) => {
        match $parser.asi() {
            Asi::ExplicitSemicolon => $parser.consume(),
            Asi::ImplicitSemicolon => {},
            Asi::NoSemicolon       => unexpected_token!($parser),
        }
    }
}

/// Return an error for current token.
#[macro_export]
macro_rules! unexpected_token {
    ($parser:ident) => ({
        // return Err($parser.lexer.invalid_token())
        use parser::error::Handle;

        let err = $parser.lexer.invalid_token();

        return Handle::handle_error($parser, err);
    });
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_expr {
    ($module:expr, $expr:expr) => ({
        let mut body = $module.body().iter();

        match **body.next().unwrap() {
            Statement::Expression {
                ref expression
            } => assert_eq!(expression.item, $expr),
            _ => panic!("Statement isn't an expression!")
        }

        assert_eq!(body.next(), None);
    })
}
