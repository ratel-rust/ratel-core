use lexicon::Token;
use lexicon::Token::*;
use lexicon::KeywordKind::*;
use lexicon::ReservedKind::*;
use lexicon::CompareKind::*;
use lexicon::OperatorKind::*;
use tokenizer::Tokenizer;
use literals::*;

#[derive(Debug)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

#[derive(Debug)]
pub enum Expression {
    Literal(LiteralValue),
}

#[derive(Debug)]
pub struct VariableDeclarator {
    id: String,
    value: Expression,
}

#[derive(Debug)]
pub struct Program {
    body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration {
        kind: VariableDeclarationKind,
        declarations: Vec<VariableDeclarator>,
    },
    Comment(String),
    BlockComment(String),
}

fn variable_declaration(
    kind: VariableDeclarationKind,
    tokenizer: &mut Tokenizer
) -> Statement {
    let mut declarations: Vec<VariableDeclarator> = Vec::new();

    'outer: loop {
        let id = match tokenizer.next() {
            Some(Identifier(id)) => id,
            _                    => panic!("WAT"),
        };
        match tokenizer.next() {
            Some(Assign)         => {},
            _                    => panic!("WAT2")
        }
        let expression = match tokenizer.next() {
            Some(Literal(kind))  => Expression::Literal(kind),
            _                    => panic!("WAT3")
        };

        declarations.push(
            VariableDeclarator {
                id: id,
                value: expression,
            }
        );

        let mut new_line = false;

        loop {
            match tokenizer.next() {
                Some(LineTermination) => new_line = true,
                Some(Semicolon)       => break 'outer,
                Some(Comma)           => continue 'outer,
                _                     => {
                    if new_line {
                        break 'outer;
                    } else {
                        panic!("WUUUUT!");
                    }
                }
            }
        }
    }

    Statement::VariableDeclaration {
        kind: kind,
        declarations: declarations,
    }
}

fn statement(tokenizer: &mut Tokenizer) -> Option<Statement> {
    if let Some(token) = tokenizer.next() {
        return match token {
            LineTermination       => statement(tokenizer),
            Comment(comment)      => Some(
                Statement::Comment(comment)
            ),
            BlockComment(comment) => Some (
                Statement::BlockComment(comment)
            ),
            Keyword(Var)          => Some(variable_declaration(
                VariableDeclarationKind::Var,
                tokenizer
            )),
            Keyword(Let)          => Some(variable_declaration(
                VariableDeclarationKind::Let,
                tokenizer
            )),
            Keyword(Const)        => Some(variable_declaration(
                VariableDeclarationKind::Const,
                tokenizer
            )),
            _ => None,
        }
    }
    return None;
}

pub fn parse(source: String) -> Program {
    let mut tokenizer = Tokenizer::new(&source);
    let mut program = Program { body: Vec::new() };

    // for token in tokenizer {
    //     println!("{:?}", token);
    // }
    while let Some(statement) = statement(&mut tokenizer) {
        program.body.push(statement);
    }

    println!("{:#?}", program);

    return program;
}
