use grammar::*;

fn visit(statement: Statement) -> Statement {
    match statement.clone() {
        Statement::VariableDeclarationStatement{kind: kind, declarations: declarations } => {
            if kind == VariableDeclarationKind::Const {
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
}
