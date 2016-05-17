use grammar::*;
use grammar::Statement::*;
use grammar::Expression::*;
use grammar::ClassMember::*;
use grammar::OperatorType::*;

/// The `Transformable` trait provides an interface for instances of grammar
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

                let body = match **body {
                    BlockStatement { ref body }   => body.clone(),
                    ExpressionStatement(ref expr) => vec![
                        ReturnStatement(expr.clone())
                    ],
                    ref statement => {
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

            ArrayExpression(ref mut items) => {
                items.transform();
                return;
            },

            ObjectExpression(ref mut members) => {
                members.transform();

                let (mut computed, literal): (Vec<ObjectMember>, Vec<ObjectMember>)
                = members.drain(..).partition(|member| {
                    match *member {
                        ObjectMember::Computed { .. } => true,
                        _                             => false,
                    }
                });

                if computed.is_empty() {
                    *members = literal;
                    return;
                }

                let mut body = Vec::new();

                body.push(VariableDeclarationStatement {
                    kind: VariableDeclarationKind::Var,
                    declarators: vec![
                        VariableDeclarator {
                            name: "___".to_string(),
                            value: ObjectExpression(literal),
                        }
                    ]
                });

                for member in computed.drain(..) {
                    if let ObjectMember::Computed { key, value } = member {
                        body.push(ExpressionStatement(BinaryExpression {
                            left: Box::new(MemberExpression {
                                object: Box::new(
                                    IdentifierExpression("___".to_string()
                                )),
                                property: Box::new(
                                    MemberKey::Computed(key)
                                )
                            }),
                            operator: Assign,
                            right: Box::new(value),
                        }));
                    }
                }

                body.push(ReturnStatement(
                    IdentifierExpression("___".to_string())
                ));

                CallExpression {
                    callee: Box::new(FunctionExpression {
                        name: None,
                        params: Vec::new(),
                        body: body,
                    }),
                    arguments: Vec::new(),
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

            ArrayExpression(ref items) => items.contains_this(),

            ObjectExpression(ref members) => members.contains_this(),

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

impl Transformable for ObjectMember {
    fn transform(&mut self) {
        *self = match *self {

            ObjectMember::Shorthand {
                ref key,
            } => {
                ObjectMember::Literal {
                    key: key.clone(),
                    value: IdentifierExpression(key.clone()),
                }
            },

            ObjectMember::Literal {
                ref mut value,
                ..
            } => {
                value.transform();
                return;
            },

            ObjectMember::Computed {
                ref mut key,
                ref mut value,
            } => {
                key.transform();
                value.transform();
                return;
            },
        }
    }

    fn contains_this(&self) -> bool {
        match *self {
            ObjectMember::Literal {
                ref value,
                ..
            } => value.contains_this(),

            ObjectMember::Computed {
                ref key,
                ref value,
            } => key.contains_this() || value.contains_this(),

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

impl Transformable for VariableDeclarator {
    fn transform(&mut self) {
        self.value.transform();
    }

    fn contains_this(&self) -> bool {
        self.value.contains_this()
    }
}

impl Transformable for Statement {
    fn transform(&mut self) {
        *self = match *self {
            VariableDeclarationStatement {
                ref mut kind,
                ref mut declarators,
            } => {
                *kind = VariableDeclarationKind::Var;
                declarators.transform();
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
            VariableDeclarationStatement {
                ref declarators,
                ..
            } => declarators.contains_this(),

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
