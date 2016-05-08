use grammar::*;
use grammar::Statement::*;
use grammar::Expression::*;

/// The `Transformable` trait provides an interace for instances of grammar
/// to alter the AST, either by mutating self, or by returning a new node.
///
/// NOTE: Returning `None` means no changes are necessary!
trait Transformable {
    fn transform(&mut self) {}
}

impl Transformable for Parameter {}

impl Transformable for Expression {
    fn transform(&mut self) {
        *self = match *self {
            ArrowFunctionExpression {
                ref mut params,
                ref mut body,
            } => {
                params.transform();
                body.transform();

                // return on feature switch
                // return;

                bind_this(FunctionExpression {
                    name: None,
                    params: params.clone(),
                    body: match *body.clone() {
                        BlockStatement { body }   => body,
                        ExpressionStatement(expr) => vec![
                            ReturnStatement(expr)
                        ],
                        statement => {
                            panic!("Invalid arrow function body {:#?}", statement);
                        }
                    },
                })
            },

            CallExpression {
                ref mut callee,
                ref mut arguments,
            } => {
                callee.transform();
                arguments.transform();
                return;
            },

            _ => return,
        }
    }
}

impl<T: Transformable> Transformable for Vec<T> {
    fn transform(&mut self) {
        for item in self.iter_mut() {
            item.transform();
        }
    }
}

impl Transformable for Statement {
    fn transform(&mut self) {
        match *self {
            VariableDeclarationStatement {
                ref mut kind,
                ..
            } => {
                *kind = VariableDeclarationKind::Var;
            },

            ExpressionStatement(ref mut expression) => {
                expression.transform();
            },

            IfStatement {
                ref mut test,
                ref mut consequent,
                ref mut alternate,
                ..
            } => {
                test.transform();
                consequent.transform();
                if let Some(ref mut alternate) = *alternate {
                    alternate.transform();
                }
            },

            BlockStatement {
                ref mut body,
            } => {
                body.transform();
            },

            _ => {},
        }
    }
}

fn bind_this(function: Expression) -> Expression {
    CallExpression {
        callee: Box::new(MemberExpression {
            object: Box::new(function),
            property: Box::new(MemberKey::Literal("bind".to_string())),
        }),
        arguments: vec![ThisExpression]
    }
}

pub fn transform(program: &mut Program) {
    program.body.transform();
}
