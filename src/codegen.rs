use grammar::*;

pub fn generate_code(program: Program) -> Result<String, Vec<String>> {
    let mut resulting_program = String::new();
    let mut errors = Vec::new();

    for stmt in program.body {
        match visit(stmt) {
            Ok(p) => resulting_program.push_str(&p),
            Err(err) => errors.push(err)
        }
    }
    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(resulting_program)
    }
}

fn visit(statement: Statement) -> Result<String, String> {
    match statement {
        Statement::VariableDeclarationStatement{kind: kind, declarations: declarations } => {
            var_declaration(kind, declarations)
        },
        _ => Err(format!("Unknown tree node {:?}",statement))
    }
}

fn var_declaration(kind: VariableDeclarationKind, declarations: Vec<(String, Expression)>) -> Result<String, String> {
    match kind {
        VariableDeclarationKind::Var | VariableDeclarationKind::Const => {
            let ref variable_name = declarations[0].0;
            let value  = declarations[0].1.clone();
            Ok(format!("var {} = {};", variable_name, literal_value(value).unwrap()))
        },
        VariableDeclarationKind::Let => Err("Unsupported statement: let".into()),
    }
}

fn literal_value(expression: Expression) -> Result<String, String> {
    match expression {
        Expression::LiteralExpression(literal) => {
            Ok(literal_to_string(literal).unwrap())
        },
        _ => Err(format!("unsupported expression type: {:?}", expression))
    }
}

fn literal_to_string(literal: LiteralValue) -> Result<String, String> {
    match literal {
        LiteralUndefined => Ok("undefined".into()),
        LiteralNull => Ok("null".into()),
        LiteralTrue => Ok("true".into()),
        LiteralFalse => Ok("false".into()),
        LiteralInteger(num) => Ok(num.to_string()),
        LiteralFloat(num) => Ok(num.to_string()),
        LiteralString(st) => Ok(format!("\"{}\"", st)),
        _ => Err("Invalid literal".into())
    }
}
