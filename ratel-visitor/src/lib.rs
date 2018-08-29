#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate ratel;

use ratel::ast::expression::*;
use ratel::ast::statement::*;
use ratel::ast::{ExpressionList, ExpressionNode, StatementList, StatementNode};
use ratel::ast::{Identifier, Literal, Node, NodeList, Pattern};

use ratel::Module;

mod expression;
mod function;
mod statement;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScopeKind {
    Function,
    Block,
}

// Like Batman!
pub type NoParent = ();

#[allow(unused_variables)]
pub trait Visitor<'ast> {
    /// after visiting a node, add it to the stack of parents
    fn push_parent(&mut self, node: ParentNode<'ast>) {}
    /// after visiting a node's children, pop it from the stack of parents
    fn pop_parent(&mut self) {}

    /// Enters a new statement list (program body, block body, switch case, etc.)
    fn on_statement_list(&mut self, body: StatementList<'ast>) {}

    /// Entered a new scope
    fn on_enter_scope(&mut self, kind: ScopeKind) {}

    /// Leave the current scope
    fn on_leave_scope(&mut self) {}

    /// A reference has been used within the current scope
    fn on_reference_use(&mut self, ident: &Identifier<'ast>) {}

    /// A reference has been declared within the current scope
    fn on_reference_declaration(&mut self, ident: &Identifier<'ast>) {}

    // expressions
    fn on_this_expression(&mut self, node: &'ast ExpressionNode<'ast>) {}
    fn on_identifier_expression(&mut self, item: &Identifier<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_literal_expression(&mut self, item: &Literal<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_sequence_expression(&mut self, item: &SequenceExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_array_expression(&mut self, item: &ArrayExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_member_expression(&mut self, item: &MemberExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_computed_member_expression(&mut self, item: &ComputedMemberExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_meta_property(&mut self, item: &MetaPropertyExpression<'ast>, node: &ExpressionNode<'ast>) {}
    fn on_call_expression(&mut self, item: &CallExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_binary_expression(&mut self, item: &BinaryExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_prefix_expression(&mut self, item: &PrefixExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_postfix_expression(&mut self, item: &PostfixExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_conditional_expression(&mut self, item: &ConditionalExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_template_literal(&mut self, item: &TemplateLiteral<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_tagged_template_expression(&mut self, item: &TaggedTemplateExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_spread_expression(&mut self, item: &SpreadExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_arrow_expression(&mut self, item: &ArrowExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_object_expression(&mut self, item: &ObjectExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_function_expression(&mut self, item: &FunctionExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}
    fn on_class_expression(&mut self, item: &ClassExpression<'ast>, node: &'ast ExpressionNode<'ast>) {}

    // statements
    fn on_expression_statement(&mut self, item: &'ast ExpressionNode<'ast>, node: &'ast StatementNode<'ast>) {}
    fn on_declaration_statement(&mut self, item: &DeclarationStatement, node: &'ast StatementNode<'ast>) {}
    fn on_return_statement(&mut self, item: &ReturnStatement, node: &'ast StatementNode<'ast>) {}
    fn on_break_statement(&mut self, item: &BreakStatement, node: &'ast StatementNode<'ast>) {}
    fn on_continue_statement(&mut self, item: &ContinueStatement, node: &'ast StatementNode<'ast>) {}
    fn on_throw_statement(&mut self, item: &ThrowStatement, node: &'ast StatementNode<'ast>) {}
    fn on_if_statement(&mut self, item: &IfStatement, node: &'ast StatementNode<'ast>) {}
    fn on_while_statement(&mut self, item: &WhileStatement, node: &'ast StatementNode<'ast>) {}
    fn on_do_statement(&mut self, item: &DoStatement, node: &'ast StatementNode<'ast>) {}
    fn on_for_statement(&mut self, item: &ForStatement, node: &'ast StatementNode<'ast>) {}
    fn on_for_in_statement(&mut self, item: &ForInStatement, node: &'ast StatementNode<'ast>) {}
    fn on_for_of_statement(&mut self, item: &ForOfStatement, node: &'ast StatementNode<'ast>) {}
    fn on_try_statement(&mut self, item: &TryStatement, node: &'ast StatementNode<'ast>) {}
    fn on_block_statement(&mut self, item: &BlockStatement<'ast>, node: &'ast StatementNode<'ast>) {}
    fn on_labeled_statement(&mut self, item: &LabeledStatement, node: &'ast StatementNode<'ast>) {}
    fn on_switch_statement(&mut self, item: &SwitchStatement, node: &'ast StatementNode<'ast>) {}
    fn on_function_statement(&mut self, item: &FunctionStatement<'ast>, node: &'ast StatementNode<'ast>) {}
    fn on_class_statement(&mut self, item:&ClassStatement<'ast>, node: &'ast StatementNode<'ast>) {}
}

#[derive(Debug, Clone, Copy)]
pub enum ParentNode<'ast> {
    Statement(&'ast StatementNode<'ast>),
    Expression(&'ast ExpressionNode<'ast>),
}

impl<'ast> From<&'ast StatementNode<'ast>> for ParentNode<'ast> {
    #[inline]
    fn from(node: &'ast StatementNode<'ast>) -> ParentNode<'ast> {
        ParentNode::Statement(node)
    }
}

impl<'ast> From<&'ast ExpressionNode<'ast>> for ParentNode<'ast> {
    #[inline]
    fn from(node: &'ast ExpressionNode<'ast>) -> ParentNode<'ast> {
        ParentNode::Expression(node)
    }
}

pub trait Visitable<'ast>: 'ast {
    type Parent;

    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>;
}

impl<'ast> Visitable<'ast> for Module<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        let body = self.body();
        for item in body {
            item.visit_with(visitor);
        }
    }
}

impl<'ast> Visitable<'ast> for Pattern<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        match *self {
            Pattern::Void => {}
            Pattern::Identifier(ref ident) => visitor.on_reference_declaration(ident),
            Pattern::ObjectPattern { ref properties } => {
                properties.visit_with(visitor);
            }
            Pattern::ArrayPattern { ref elements } => {
                elements.visit_with(visitor);
            }
            Pattern::RestElement { ref argument } => {
                argument.visit_with(visitor);
            }
            Pattern::AssignmentPattern {
                ref left,
                ref right,
            } => {
                left.visit_with(visitor);
                right.visit_with(visitor);
            }
        }
    }
}

impl<'ast> Visitable<'ast> for PropertyKey<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        match *self {
            PropertyKey::Computed(ref expression) => expression.visit_with(visitor),
            PropertyKey::Literal(_) | PropertyKey::Binary(_) => {}
        }
    }
}

impl<'ast> Visitable<'ast> for Property<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        match *self {
            Property::Shorthand(ref ident) => visitor.on_reference_use(ident),
            Property::Literal { ref key, ref value } => {
                key.visit_with(visitor);
                value.visit_with(visitor);
            }
            Property::Method { ref key, ref value } => {
                key.visit_with(visitor);
                value.visit_with(visitor);
            },
            Property::Spread { ref argument } => {
                argument.visit_with(visitor);
            }
        }
    }
}

impl<'ast, T> Visitable<'ast> for Option<T>
where
    T: Visitable<'ast>,
{
    type Parent = T::Parent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        if let Some(ref visitable) = *self {
            visitable.visit_with(visitor);
        }
    }
}

// Requiring that `Parent = Node<'ast, T>` means that we avoid having
// a default implementation for (Expression|Statement)(Node|List)
impl<'ast, T> Visitable<'ast> for Node<'ast, T>
where
    T: Visitable<'ast, Parent = Node<'ast, T>>,
{
    type Parent = NoParent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        self.item.visit_with(visitor);
    }
}

impl<'ast, T> Visitable<'ast> for NodeList<'ast, T>
where
    T: Visitable<'ast, Parent = Node<'ast, T>>,
{
    type Parent = NoParent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        for item in self {
            item.visit_with(visitor);
        }
    }
}

impl<'ast> Visitable<'ast> for ExpressionList<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        for node in self.iter() {
            node.visit_with(visitor);
        }
    }
}

impl<'ast> Visitable<'ast> for StatementList<'ast> {
    type Parent = NoParent;

    #[inline]
    fn visit_with<V>(&'ast self, visitor: &mut V)
    where
        V: Visitor<'ast>,
    {
        visitor.on_statement_list(*self);
        for node in self.iter() {
            node.visit_with(visitor);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ratel::parse;
    use ScopeKind::*;

    struct ScopeTest<'ast> {
        depth: i32,
        max_depth: i32,
        scopes: Vec<ScopeKind>,
        used_vars: Vec<(&'ast str, i32)>,
        declared_vars: Vec<(&'ast str, i32)>,
    }

    impl<'ast> ScopeTest<'ast> {
        fn new() -> Self {
            ScopeTest {
                depth: 0,
                max_depth: 0,
                scopes: Vec::new(),
                used_vars: Vec::new(),
                declared_vars: Vec::new(),
            }
        }
    }

    impl<'ast> Visitor<'ast> for ScopeTest<'ast> {
        fn on_enter_scope(&mut self, kind: ScopeKind) {
            self.scopes.push(kind);
            self.depth += 1;
            self.max_depth = self.max_depth.max(self.depth);
        }

        fn on_leave_scope(&mut self) {
            self.depth -= 1;
        }

        fn on_reference_use(&mut self, ident: &Identifier<'ast>) {
            self.used_vars.push((*ident, self.depth));
        }

        fn on_reference_declaration(&mut self, ident: &Identifier<'ast>) {
            self.declared_vars.push((*ident, self.depth));
        }
    }

    #[test]
    fn keeps_track_of_blocks() {
        let module = parse("{{{}}}").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[Block, Block, Block]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 3);
        assert_eq!(visitor.used_vars, &[]);
        assert_eq!(visitor.declared_vars, &[]);
    }

    #[test]
    fn keeps_track_of_declarations() {
        let module = parse("let foo; const bar = 42, doge;").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 0);
        assert_eq!(visitor.used_vars, &[]);
        assert_eq!(visitor.declared_vars, &[("foo", 0), ("bar", 0), ("doge", 0)]);
    }

    #[test]
    fn keeps_track_of_declarations_at_the_correct_depth() {
        let module = parse("let foo; { let foo; { let foo; }}").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[Block, Block]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 2);
        assert_eq!(visitor.used_vars, &[]);
        assert_eq!(visitor.declared_vars, &[("foo", 0), ("foo", 1), ("foo", 2)]);
    }

    #[test]
    fn keeps_track_of_uses() {
        let module = parse("doge = to + the + moon").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 0);
        assert_eq!(
            visitor.used_vars,
            &[("doge", 0), ("to", 0), ("the", 0), ("moon", 0)]
        );
        assert_eq!(visitor.declared_vars, &[]);
    }

    #[test]
    fn keeps_track_of_uses_at_the_correct_depth() {
        let module = parse("doge; { to; { the; { moon; }}}").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[Block, Block, Block]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 3);
        assert_eq!(
            visitor.used_vars,
            &[("doge", 0), ("to", 1), ("the", 2), ("moon", 3)]
        );
        assert_eq!(visitor.declared_vars, &[]);
    }

    #[test]
    fn function_and_class_are_declarations() {
        let module = parse("function foo() {} class Bar {}").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[Function]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 1);
        assert_eq!(visitor.used_vars, &[]);
        assert_eq!(visitor.declared_vars, &[("foo", 0), ("Bar", 0)]);
    }

    #[test]
    fn function_and_class_expressions_are_not_declarations() {
        let module = parse("(function foo() {}); (class Bar {});").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[Function]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 1);
        assert_eq!(visitor.used_vars, &[]);
        assert_eq!(visitor.declared_vars, &[]);
    }

    #[test]
    fn empty_class_has_no_scope() {
        let module = parse("class Doge {}").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 0);
        assert_eq!(visitor.used_vars, &[]);
        assert_eq!(visitor.declared_vars, &[("Doge", 0)]);
    }

    #[test]
    fn functions_and_object_methods_are_scopes() {
        let module = parse(
            r"
            function doge() {
                foo;

                return {
                    baz() {
                        bar;
                    }
                };
            }
        ",
        ).unwrap();

        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[Function, Function]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 2);
        assert_eq!(visitor.used_vars, &[("foo", 1), ("bar", 2)]);
        assert_eq!(visitor.declared_vars, &[("doge", 0)]);
    }

    #[test]
    fn object_property_shorthand_is_a_use() {
        let module = parse("const doge = { to, the, moon };").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 0);
        assert_eq!(visitor.used_vars, &[("to", 0), ("the", 0), ("moon", 0)]);
        assert_eq!(visitor.declared_vars, &[("doge", 0)]);
    }

    #[test]
    fn function_params_are_declarations() {
        let module = parse("function doge(to, the) { const moon; }").unwrap();
        let mut visitor = ScopeTest::new();

        module.visit_with(&mut visitor);

        assert_eq!(visitor.scopes, &[Function]);
        assert_eq!(visitor.depth, 0);
        assert_eq!(visitor.max_depth, 1);
        assert_eq!(visitor.used_vars, &[]);
        assert_eq!(
            visitor.declared_vars,
            &[("doge", 0), ("to", 1), ("the", 1), ("moon", 1)]
        );
    }

    struct ParentsTest<'ast> {
        count: u32,
        parents: Vec<ParentNode<'ast>>,
    }

   impl<'ast> ParentsTest<'ast> {
        fn new() -> ParentsTest<'ast> {
            ParentsTest {
                count: 0,
                parents: Vec::new(),
            }
        }
    }

    impl<'ast> Visitor<'ast> for ParentsTest<'ast> {
        fn push_parent(&mut self, node: ParentNode<'ast>) {
            self.count += 1;
            self.parents.push(node);
        }

        fn pop_parent(&mut self) {
            assert_eq!(self.parents.pop().is_some(), true);
        }
    }

    #[test]
    fn should_track_1_parent_for_every_statement_or_expression_node() {
        let module = parse("{{{1+2}}}").unwrap();
        let mut visitor = ParentsTest::new();

        module.visit_with(&mut visitor);

        // 3 BlockStatements + 1 ExpressionStatement + 1 BinaryExpression = 5 Parents tracked
        // The 2 Literals don't have children, therefore they aren't tracked
        assert_eq!(visitor.count, 5);
        assert_eq!(visitor.parents.len(), 0);
    }
}
