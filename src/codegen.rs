use grammar::*;
use grammar::Statement::*;
use grammar::Expression::*;

trait Codegen {
    fn stringify(&self) -> String;
}

impl Codegen for LiteralValue {
    fn stringify(&self) -> String {
        match *self {
            LiteralUndefined        => "undefined".into(),
            LiteralNull             => "null".into(),
            LiteralTrue             => "true".into(),
            LiteralFalse            => "false".into(),
            LiteralInteger(ref num) => num.to_string(),
            LiteralFloat(ref num)   => num.to_string(),
            LiteralString(ref st)   => format!("\"{}\"", st)
        }
    }
}

impl Codegen for Expression {
    fn stringify(&self) -> String {
        match *self {
            IdentifierExpression(ref ident) => ident.clone(),
            LiteralExpression(ref literal)  => literal.stringify(),
            _ => '💀'.to_string(),
        }
    }
}

impl Codegen for VariableDeclarationKind {
    fn stringify(&self) -> String {
        match *self {
            VariableDeclarationKind::Var   => "var".into(),
            VariableDeclarationKind::Let   => "let".into(),
            VariableDeclarationKind::Const => "const".into(),
        }
    }
}

impl Codegen for Statement {
    fn stringify(&self) -> String {
        match *self {

            VariableDeclarationStatement {
                ref kind,
                ref declarations,
            } => var_declaration(kind, declarations),

            IfStatement {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                let mut code = format!("if ({}) {}",
                    test.stringify(),
                    consequent.stringify()
                );

                if let &Some(ref alternate) = alternate {
                    code.push_str(&format!(" else {}", alternate.stringify()));
                };

                code
            },

            BlockStatement {
                ref body
            } => {
                let mut code = '{'.to_string();
                for stmt in body {
                    code.push_str(&stmt.stringify());
                    code.push('\n');
                }
                code.push('}');
                code
            },

            _ => '💀'.to_string(),
        }
    }
}

pub fn generate_code(mut program: Program) -> Result<String, Vec<String>> {
    let mut resulting_program = String::new();
    // let mut errors = Vec::new();

    for stmt in program.body {
        resulting_program.push_str(&stmt.stringify());
        resulting_program.push('\n');
    }

    Ok(resulting_program)
}

fn var_declaration(kind: &VariableDeclarationKind, declarations: &Vec<(String, Expression)>) -> String {
    format!("{} {};", kind.stringify(), declarations.into_iter()
        .map(|&(ref name, ref value)| {
            format!("{} = {}", name, value.stringify())
        })
        .collect::<Vec<String>>()
        .join(", ")
    )
}
