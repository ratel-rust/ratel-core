use grammar::*;

fn visit(statement: Statement) -> Statement {
    match statement.clone() {
        Statement::IfStatement{test: t, consequent: consequent, alternate: alternate} => {
            Statement::IfStatement{
                test: t,
                consequent: Box::new(visit(*consequent)),
                alternate: alternate
            }
        },
        Statement::BlockStatement{ body: body } => {
            let mut new_body = Vec::new();
            for statement in body {
                new_body.push(visit(statement));
            }
            Statement::BlockStatement {
                body: new_body
            }
        },
        Statement::VariableDeclarationStatement{kind: kind, declarations: declarations } => {
            if kind == VariableDeclarationKind::Const || kind == VariableDeclarationKind::Let {
                Statement::VariableDeclarationStatement {
                    kind: VariableDeclarationKind::Var,
                    declarations: declarations
                }
            } else {
                statement
            }
        },
        _ => statement
    }
}

pub fn traverse(program: Program) -> Program {
    let mut new_statements = Vec::new();

    for statement in program.body {
        new_statements.push(visit(statement));
    }

    Program {
        body: new_statements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::*;
    use grammar::*;

    #[test]
    fn transform_const_assignment() {
        let input_program = parse("const foo = 42;".into());
        let output_program = traverse(input_program);
        let statement = output_program.body.first().unwrap().clone();
        let success = match statement {
            Statement::VariableDeclarationStatement{kind: kind, declarations: _declarations } => {
                kind == VariableDeclarationKind::Var
            },
            _ => false
        };
        assert!(success);
    }

    #[test]
    fn transform_simple_let_assignment() {
        let input_program = parse("let foo = 42;".into());
        let output_program = traverse(input_program);
        let statement = output_program.body.first().unwrap().clone();
        let success = match statement {
            Statement::VariableDeclarationStatement{kind: kind, declarations: _declarations } => {
                kind == VariableDeclarationKind::Var
            },
            _ => false
        };
        assert!(success);
    }

    #[test]
    fn transform_const_inside_of_if() {
        let input_program = "
        if(true) {
            const pi = 3.14;
        }
        ";
        let output_program = traverse(parse(input_program.into()));
        let statement = output_program.body.first().unwrap().clone();
        match statement {
            Statement::IfStatement{test: _exp, consequent: stmt, alternate: _stmt} => {
                match *stmt.clone() {
                    Statement::BlockStatement{body: body} => {
                        let first_body_stmt = body.first().unwrap().clone();
                        match first_body_stmt {
                            Statement::VariableDeclarationStatement{kind: kind, declarations: _declarations } => {
                                assert_eq!(kind, VariableDeclarationKind::Var);
                            },
                            _ => assert!(false,
                                    format!("Expected Variable Declaration, Received {:?}", first_body_stmt))
                        }
                    },
                    _ => assert!(false,
                                 format!("Expected Block statement, Got {:?}", *stmt))
                }
            },
            _ => assert!(false, "received invalid statement. Expected if")
        }
    }
}
