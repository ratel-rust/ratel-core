use grammar::*;
use grammar::Statement::*;
use grammar::Expression::*;
use grammar::ClassMember::*;

/// The `Transformable` trait provides an interace for instances of grammar
/// to alter the AST, either by mutating self, or by returning a new node.
///
/// NOTE: Returning `None` means no changes are necessary!
trait Transformable {
    fn transform(&mut self) {}

    fn contains_this(&self) -> bool {
        false
    }
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

                let body = match *body.clone() {
                    BlockStatement { body }   => body,
                    ExpressionStatement(expr) => vec![
                        ReturnStatement(expr)
                    ],
                    statement => {
                        panic!("Invalid arrow function body {:#?}", statement);
                    }
                };

                let bind = body.contains_this();

                let function = FunctionExpression {
                    name: None,
                    params: params.clone(),
                    body: body,
                };

                if bind {
                    bind_this(function)
                } else {
                    function
                }
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

    fn contains_this(&self) -> bool {
        match *self {
            ThisExpression => true,

            MemberExpression {
                ref object,
                ..
            } => object.contains_this(),

            CallExpression {
                ref callee,
                ref arguments,
            } => callee.contains_this() || arguments.contains_this(),

            _ => false,
        }
    }
}

impl Transformable for ClassMember {
    fn transform(&mut self) {
        match *self {
            ClassConstructor {
                ref mut params,
                ref mut body,
            } => {
                params.transform();
                body.transform();
            },

            ClassMethod {
                ref mut params,
                ref mut body,
                ..
            } => {
                params.transform();
                body.transform();
            },

            ClassProperty {
                ref mut value,
                ..
            } => {
                value.transform();
            }
        }
    }
}

impl Transformable for Statement {
    fn transform(&mut self) {
        *self = match *self {
            VariableDeclarationStatement {
                ref mut kind,
                ..
            } => {
                *kind = VariableDeclarationKind::Var;
                return;
            },

            ExpressionStatement(ref mut expression) => {
                expression.transform();
                return;
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
                return;
            },

            BlockStatement {
                ref mut body,
            } => {
                body.transform();
                return;
            },

            ClassStatement {
                ref mut body,
                ..
            } => {
                body.transform();
                return;
            }

            _ => return,
        }
    }

    fn contains_this(&self) -> bool {
        match *self {
            ExpressionStatement(ref expression) => expression.contains_this(),

            ReturnStatement(ref expression) => expression.contains_this(),

            _ => false,
        }
    }
}

impl<T: Transformable> Transformable for Vec<T> {
    fn transform(&mut self) {
        for item in self.iter_mut() {
            item.transform();
        }
    }

    fn contains_this(&self) -> bool {
        for item in self {
            if item.contains_this() {
                return true;
            }
        }
        return false;
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
