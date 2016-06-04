use grammar::*;
use grammar::Statement::*;
use grammar::Expression::*;
use grammar::ClassMember::*;
use grammar::OperatorType::*;

pub struct Settings {
    pub transform_block_scope: bool,
    pub transform_arrow: bool,
    pub transform_object: bool,
    pub transform_exponentation: bool,
}

#[inline(always)]
fn bind_this(function: Expression) -> Expression {
    Expression::call(Expression::member(function, "bind"), vec![ThisExpression])
}

impl Settings {
    pub fn target_es5() -> Settings {
        let mut settings = Settings::target_es2015();

        settings.transform_block_scope = true;
        settings.transform_arrow = true;
        settings.transform_object = true;

        settings
    }

    pub fn target_es2015() -> Settings {
        let mut settings = Settings::no_transform();

        settings.transform_exponentation = true;

        settings
    }

    pub fn no_transform() -> Settings {
        Settings {
            transform_block_scope: false,
            transform_arrow: false,
            transform_object: false,
            transform_exponentation: false,
        }
    }
}

/// The `Transformable` trait provides an interface for instances of grammar
/// to alter the AST, either by mutating self, or by returning a new node.
///
/// NOTE: Returning `None` means no changes are necessary!
trait Transformable {
    fn transform(&mut self, _: &Settings) {}

    fn contains_this(&self) -> bool {
        false
    }
}

impl Transformable for Parameter {}

impl Transformable for Expression {
    fn transform(&mut self, settings: &Settings) {
        *self = match *self {
            ArrowFunctionExpression {
                ref mut params,
                ref mut body,
            } => {
                params.transform(settings);
                body.transform(settings);

                // transformation flag check
                if !settings.transform_arrow {
                    return;
                }

                let body = match **body {
                    BlockStatement { ref mut body } => body.split_off(0),
                    ExpressionStatement(ref expr)   => vec![
                        ReturnStatement {
                            value: Some(expr.clone())
                        }
                    ],
                    ref statement => {
                        panic!("Invalid arrow function body {:#?}", statement);
                    }
                };

                let bind = body.contains_this();

                let function = FunctionExpression {
                    name: None,
                    params: params.split_off(0),
                    body: body,
                };

                if bind {
                    bind_this(function)
                } else {
                    function
                }
            },

            ArrayExpression(ref mut items) => {
                items.transform(settings);
                return;
            },

            ObjectExpression(ref mut members) => {
                members.transform(settings);

                // transformation flag check
                if !settings.transform_object {
                    return;
                }

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
                            value: Some(ObjectExpression(literal)),
                        }
                    ]
                });

                for member in computed.drain(..) {
                    if let ObjectMember::Computed { key, value } = member {
                        body.push(ExpressionStatement(BinaryExpression {
                            left: Box::new(MemberExpression {
                                object: Box::new(Expression::ident("___")),
                                property: Box::new(
                                    MemberKey::Computed(key)
                                )
                            }),
                            operator: Assign,
                            right: Box::new(value),
                        }));
                    }
                }

                body.push(ReturnStatement {
                    value: Some(Expression::ident("___"))
                });

                Expression::call(FunctionExpression {
                    name: None,
                    params: Vec::new(),
                    body: body,
                }, Vec::new())
            },

            CallExpression {
                ref mut callee,
                ref mut arguments,
            } => {
                callee.transform(settings);
                arguments.transform(settings);
                return;
            },

            BinaryExpression {
                ref mut left,
                ref mut operator,
                ref mut right,
            } => {
                left.transform(settings);
                right.transform(settings);

                if !settings.transform_exponentation {
                    return;
                }

                match *operator {
                    Exponent => Expression::call(
                        Expression::member(Expression::ident("Math"), "pow"),
                        vec![*left.clone(), *right.clone()]
                    ),

                    ExponentAssign => {
                        *operator = Assign;
                        *right = Box::new(Expression::call(
                            Expression::member(Expression::ident("Math"), "pow"),
                            vec![*left.clone(), *right.clone()]
                        ));
                        return;
                    },

                    _ => return,
                }
            }

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

            BinaryExpression {
                ref left,
                ref right,
                ..
            } => left.contains_this() || right.contains_this(),

            _ => false,
        }
    }
}

impl Transformable for ObjectMember {
    fn transform(&mut self, settings: &Settings) {
        *self = match *self {

            ObjectMember::Shorthand {
                ref key,
            } => {
                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Literal {
                    key: key.clone(),
                    value: IdentifierExpression(key.clone()),
                }
            },

            ObjectMember::Literal {
                ref mut value,
                ..
            } => {
                value.transform(settings);
                return;
            },

            ObjectMember::Computed {
                ref mut key,
                ref mut value,
            } => {
                key.transform(settings);
                value.transform(settings);
                return;
            },

            ObjectMember::Method {
                ref mut name,
                ref mut params,
                ref mut body,
            } => {
                body.transform(settings);
                params.transform(settings);

                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Literal {
                    key: name.clone(),
                    value: FunctionExpression {
                        name: Some(name.clone()),
                        params: params.split_off(0),
                        body: body.split_off(0),
                    }
                }
            },

            ObjectMember::ComputedMethod {
                ref mut name,
                ref mut params,
                ref mut body,
            } => {
                body.transform(settings);
                params.transform(settings);

                // transformation flag check
                if !settings.transform_object {
                    return;
                }

                ObjectMember::Computed {
                    key: name.clone(),
                    value: FunctionExpression {
                        name: None,
                        params: params.split_off(0),
                        body: body.split_off(0),
                    }
                }
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
    fn transform(&mut self, settings: &Settings) {
        match *self {
            Constructor {
                ref mut params,
                ref mut body,
            } => {
                params.transform(settings);
                body.transform(settings);
            },

            Method {
                ref mut params,
                ref mut body,
                ..
            } => {
                params.transform(settings);
                body.transform(settings);
            },

            Property {
                ref mut value,
                ..
            } => {
                value.transform(settings);
            }
        }
    }
}

impl Transformable for VariableDeclarator {
    fn transform(&mut self, settings: &Settings) {
        match self.value {
            Some(ref mut expression) => expression.transform(settings),
            _                        => {},
        }
    }

    fn contains_this(&self) -> bool {
        match self.value {
            Some(ref expression) => expression.contains_this(),
            _                    => false,
        }
    }
}

impl Transformable for Statement {
    fn transform(&mut self, settings: &Settings) {
        *self = match *self {
            BlockStatement {
                ref mut body,
            } => {
                body.transform(settings);
                return;
            },

            LabeledStatement {
                ref mut body,
                ..
            } => {
                body.transform(settings);
                return;
            },

            VariableDeclarationStatement {
                ref mut kind,
                ref mut declarators,
            } => {
                declarators.transform(settings);

                // transformation flag check
                if !settings.transform_block_scope {
                    return;
                }

                *kind = VariableDeclarationKind::Var;
                return;
            },

            ExpressionStatement(ref mut expression) => {
                expression.transform(settings);
                return;
            },

            IfStatement {
                ref mut test,
                ref mut consequent,
                ref mut alternate,
                ..
            } => {
                test.transform(settings);
                consequent.transform(settings);
                if let Some(ref mut alternate) = *alternate {
                    alternate.transform(settings);
                }
                return;
            },

            ClassStatement {
                ref mut body,
                ..
            } => {
                body.transform(settings);
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

            ReturnStatement {
                value: Some(ref expression)
            } => expression.contains_this(),

            _ => false,
        }
    }
}

impl<T: Transformable> Transformable for Vec<T> {
    fn transform(&mut self, settings: &Settings) {
        for item in self.iter_mut() {
            item.transform(settings);
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

pub fn transform(program: &mut Program, settings: Settings) {
    program.body.transform(&settings);
}
