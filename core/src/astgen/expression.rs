use serde::ser::{Serialize, Serializer, SerializeStruct};
use ast;
use ast::{ExpressionPtr, Expression, Loc, OperatorKind};
use serde_json;
use astgen::function::ClassBody;
use astgen::value::TemplateElement;
use astgen::value::TemplateLiteral;
use astgen::statement::BlockStatement;
use astgen::SerializeInLoc;

#[derive(Debug)]

struct TaggedTemplateExpression<'ast> {
    tag: ExpressionPtr<'ast>,
    quasi: Loc<TemplateLiteral<'ast>>,
}

impl<'ast> SerializeInLoc for TaggedTemplateExpression<'ast> {
    fn serialize<S>(&self, serializer: S) -> Result<S::SerializeStruct, S::Error>
        where S: Serializer
    {

        self.in_loc(serializer, "TaggedTemplateExpression", 2, |state| {
            state.serialize_field("tag", &self.tag)?;
            state.serialize_field("quasi", &self.quasi)
        })
    }
}

#[inline]
fn expression_type<'ast>(operator: OperatorKind, prefix: bool) -> &'static str {
    use self::OperatorKind::*;

    match operator {
        Assign              |
        AddAssign           |
        ExponentAssign      |
        MultiplyAssign      |
        DivideAssign        |
        RemainderAssign     |
        BSLAssign           |
        BSRAssign           |
        UBSRAssign          |
        BitOrAssign         |
        BitXorAssign        |
        SubtractAssign      |
        BitAndAssign        => "AssignmentExpression",
        LogicalAnd          |
        LogicalOr           => "LogicalExpression",
        Increment           |
        Decrement           => "UpdateExpression",
        Typeof              |
        Void                |
        Delete              => "UnaryExpression",
        Subtraction         |
        Addition            |
        LogicalNot          |
        BitwiseNot          => if prefix { "UnaryExpression" } else { "BinaryExpression" },
        _                   => "BinaryExpression"
    }
}

impl<'ast> Serialize for Loc<Expression<'ast>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use self::Expression::*;

        let mut state = match self.item {

            Error { .. } => panic!("Module contains errors"),

            Void => {
                return serializer.serialize_none()
            },

            This => {
                let mut state = serializer.serialize_struct("ThisExpression", 3)?;
                state.serialize_field("type", &"ThisExpression")?;
                state
            },

            Identifier(ident) => {
                let mut state = serializer.serialize_struct("Identifier", 4)?;
                state.serialize_field("type", &"Identifier")?;
                state.serialize_field("name", &ident)?;
                state
            },

            Value(value) => {
                use self::ast::Value::*;

                if let Template(_) = value {
                    return serializer.serialize_some(&value)
                }

                let mut state = serializer.serialize_struct("Literal", 4)?;
                state.serialize_field("type", &"Literal")?;

                if let RegEx(_) = value {
                    state.serialize_field("regex", &value)?;
                } else {
                    state.serialize_field("value", &value)?;
                }

                state
            },

            Sequence { body } => {
                let mut state = serializer.serialize_struct("SequenceExpression", 4)?;
                state.serialize_field("type", &"SequenceExpression")?;
                state.serialize_field("expressions", &body)?;
                state
            },

            Array { body } => {
                let mut state = serializer.serialize_struct("ArrayExpression", 4)?;
                state.serialize_field("type", &"ArrayExpression")?;
                state.serialize_field("elements", &body)?;
                state
            },

            Member { object, property } => {
                let mut state = serializer.serialize_struct("MemberExpression", 6)?;
                state.serialize_field("type", &"MemberExpression")?;
                state.serialize_field("object", &object)?;
                state.serialize_field("property", &Loc::new(property.start, property.end, Expression::Identifier(property.item)))?;
                state.serialize_field("computed", &false)?;
                state
            },

            ComputedMember { object, property } => {
                let mut state = serializer.serialize_struct("MemberExpression", 6)?;
                state.serialize_field("type", &"MemberExpression")?;
                state.serialize_field("object", &object)?;
                state.serialize_field("property", &property)?;
                state.serialize_field("computed", &true)?;
                state
            },

            Call { callee, arguments } => {
                let mut state = serializer.serialize_struct("CallExpression", 5)?;
                state.serialize_field("type", &"CallExpression")?;
                state.serialize_field("callee", &callee)?;
                state.serialize_field("arguments", &arguments)?;
                state
            },

            Binary { operator, left, right } => {
                let expr_type = expression_type(operator, false);
                let mut state = serializer.serialize_struct(expr_type, 6)?;
                state.serialize_field("type", &expr_type)?;
                state.serialize_field("operator", &operator.as_str())?;
                state.serialize_field("left", &left)?;
                state.serialize_field("right", &right)?;
                state
            },

            Prefix { operator, operand } => {
                if let OperatorKind::New = operator {
                    let mut state = serializer.serialize_struct("NewExpression", 5)?;
                    state.serialize_field("type", &"NewExpression")?;

                    match operand.item {
                        Call { callee, arguments } => {
                            state.serialize_field("callee", &callee)?;
                            state.serialize_field("arguments", &arguments)?;
                        },
                        Value(_) => {
                            let arguments: Vec<ExpressionPtr> = vec![];
                            state.serialize_field("callee", &operand)?;
                            state.serialize_field("arguments", &arguments)?;
                        },
                        _ => {
                        // FIXME
                            panic!("Unexpected token");
                        }
                    };

                    state

                } else {
                    let expr_type = expression_type(operator, true);
                    let mut state = serializer.serialize_struct(expr_type, 5)?;
                    state.serialize_field("type", &expr_type)?;
                    state.serialize_field("operator", &operator.as_str())?;
                    state.serialize_field("argument", &operand)?;
                    state.serialize_field("prefix", &true)?;
                    state
                }
            },

            Postfix { operator, operand }=> {
                let expr_type = expression_type(operator, false);
                let mut state = serializer.serialize_struct(expr_type, 5)?;
                state.serialize_field("type", &expr_type)?;
                state.serialize_field("operator", &operator.as_str())?;
                state.serialize_field("argument", &operand)?;
                state.serialize_field("prefix", &false)?;
                state
            },

            Conditional { test, consequent, alternate } => {
                let mut state = serializer.serialize_struct("ConditionalExpression", 6)?;
                state.serialize_field("type", &"ConditionalExpression")?;
                state.serialize_field("test", &test)?;
                state.serialize_field("alternate", &alternate)?;
                state.serialize_field("consequent", &consequent)?;
                state
            },

            Template { tag, expressions, quasis } => {
                let mut quasis = quasis.ptr_iter().map(|q| {
                    let element = TemplateElement { tail: false, value: q.item };
                    Loc::new(q.start, q.end, element)
                }).collect::<Vec<_>>();

                // FIXME: Sets `tail` to `true` on the last TemplateElement.
                let mut last = quasis.pop().unwrap();
                last.item.tail = true;
                quasis.push(last);

                let expressions = expressions.iter().map(|q| *q).collect::<Vec<_>>();

                let expr = Loc::new(self.start, self.end, TemplateLiteral { quasis, expressions });

                if let Some(tag) = tag {
                    let expr = TaggedTemplateExpression {
                        tag,
                        quasi: expr
                    };
                    return serializer.serialize_some(&Loc::new(self.start, self.end, expr))
                }
                return serializer.serialize_some(&expr)

            },

            Arrow { params, body } => {
                let mut state = serializer.serialize_struct("ArrowFunctionExpression", 6)?;
                state.serialize_field("type", &"ArrowFunctionExpression")?;
                state.serialize_field("id", &())?;
                state.serialize_field("params", &params)?;
                state.serialize_field("body", &body)?;
                state
            },

            Object { body } => {
                let mut state = serializer.serialize_struct("ObjectExpression", 4)?;
                state.serialize_field("type", &"ObjectExpression")?;
                state.serialize_field("properties", &body)?;
                state
            },

            Function { function } => {
                let mut state = serializer.serialize_struct("FunctionExpression", 6)?;
                state.serialize_field("type", &"FunctionExpression")?;
                state.serialize_field("id", &function.name)?;
                state.serialize_field("params", &function.params)?;

                match function.body.only_element() {
                    Some(&Loc { item: ast::Statement::Block { .. } , .. }) => {
                        state.serialize_field("body", &function.body)?;
                    },
                    _ => {
                        state.serialize_field("body", &Loc::new(self.start, self.end, BlockStatement { body: function.body }))?;
                    }
                };

                state
            },

            Class { class } => {
                let mut state = serializer.serialize_struct("ClassExpression", 6)?;
                state.serialize_field("type", &"ClassExpression")?;
                state.serialize_field("id", &class.name)?;
                state.serialize_field("superClass", &class.extends)?;
                state.serialize_field("body", &Loc::new(self.start, self.end, ClassBody { body: class.body }))?;
                state
            },
        };

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}
