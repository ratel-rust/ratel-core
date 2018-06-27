#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate ratel;

use ratel::ast::expression::*;
use ratel::ast::statement::*;
use ratel::ast::{ExpressionList, ExpressionNode, StatementList, StatementNode};
use ratel::ast::{Identifier, Literal, Node, NodeList, Pattern};

use ratel::Module;

#[macro_use]
mod build;
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

build! {
    // after visiting a node, add it to the stack of parents
    fn push_parent(node: ParentNode<'ast>);
    // after visiting a node's children, pop it from the stack of parents
    fn pop_parent();

    // Enters a new statement list (program body, block body, switch case, etc.)
    fn on_statement_list(body: StatementList<'ast>);

    // Entered a new scope
    fn on_enter_scope(kind: ScopeKind);

    // Leave the current scope
    fn on_leave_scope();

    // A reference has been used within the current scope
    fn on_reference_use(ident: &Identifier<'ast>);

    // A reference has been declared within the current scope
    fn on_reference_declaration(ident: &Identifier<'ast>);

    // expressions
    fn on_this_expression(node: &'ast ExpressionNode<'ast>);
    fn on_identifier_expression(item: &Identifier<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_literal_expression(item: &Literal<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_sequence_expression(item: &SequenceExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_array_expression(item: &ArrayExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_member_expression(item: &MemberExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_computed_member_expression(item: &ComputedMemberExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_meta_property(item: &MetaPropertyExpression<'ast>, node: &ExpressionNode<'ast>);
    fn on_call_expression(item: &CallExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_binary_expression(item: &BinaryExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_prefix_expression(item: &PrefixExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_postfix_expression(item: &PostfixExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_conditional_expression(item: &ConditionalExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_template_literal(item: &TemplateLiteral<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_tagged_template_expression(item: &TaggedTemplateExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_spread_expression(item: &SpreadExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_arrow_expression(item: &ArrowExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_object_expression(item: &ObjectExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_function_expression(item: &FunctionExpression<'ast>, node: &'ast ExpressionNode<'ast>);
    fn on_class_expression(item: &ClassExpression<'ast>, node: &'ast ExpressionNode<'ast>);

    // statements
    fn on_expression_statement(item: &'ast ExpressionNode<'ast>, node: &'ast StatementNode<'ast>);
    fn on_declaration_statement(item: &DeclarationStatement, node: &'ast StatementNode<'ast>);
    fn on_return_statement(item: &ReturnStatement, node: &'ast StatementNode<'ast>);
    fn on_break_statement(item: &BreakStatement, node: &'ast StatementNode<'ast>);
    fn on_continue_statement(item: &ContinueStatement, node: &'ast StatementNode<'ast>);
    fn on_throw_statement(item: &ThrowStatement, node: &'ast StatementNode<'ast>);
    fn on_if_statement(item: &IfStatement, node: &'ast StatementNode<'ast>);
    fn on_while_statement(item: &WhileStatement, node: &'ast StatementNode<'ast>);
    fn on_do_statement(item: &DoStatement, node: &'ast StatementNode<'ast>);
    fn on_for_statement(item: &ForStatement, node: &'ast StatementNode<'ast>);
    fn on_for_in_statement(item: &ForInStatement, node: &'ast StatementNode<'ast>);
    fn on_for_of_statement(item: &ForOfStatement, node: &'ast StatementNode<'ast>);
    fn on_try_statement(item: &TryStatement, node: &'ast StatementNode<'ast>);
    fn on_block_statement(item: &BlockStatement<'ast>, node: &'ast StatementNode<'ast>);
    fn on_labeled_statement(item: &LabeledStatement, node: &'ast StatementNode<'ast>);
    fn on_switch_statement(item: &SwitchStatement, node: &'ast StatementNode<'ast>);
    fn on_function_statement(item: &FunctionStatement<'ast>, node: &'ast StatementNode<'ast>);
    fn on_class_statement(item: &ClassStatement<'ast>, node: &'ast StatementNode<'ast>);
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

pub trait ParentTrackingContext<'ast> {
    #[inline]
    fn push_parent(&mut self, ParentNode<'ast>) {}

    #[inline]
    fn pop_parent(&mut self) -> Option<ParentNode<'ast>> {
        None
    }

    #[inline]
    fn get_parent(&mut self) -> Option<ParentNode<'ast>> {
        None
    }
}

pub trait Visitable<'ast>: 'ast {
    type Parent;

    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>;
}

impl<'ast> Visitable<'ast> for Module<'ast> {
    type Parent = NoParent;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        let body = self.body();
        for item in body {
            item.traverse(visitor, ctx);
        }
    }
}

impl<'ast> Visitable<'ast> for Pattern<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            Pattern::Void => {}
            Pattern::Identifier(ref ident) => visitor.on_reference_declaration(ident, ctx),
            Pattern::ObjectPattern { ref properties } => {
                properties.traverse(visitor, ctx);
            }
            Pattern::ArrayPattern { ref elements } => {
                elements.traverse(visitor, ctx);
            }
            Pattern::RestElement { ref argument } => {
                argument.traverse(visitor, ctx);
            }
            Pattern::AssignmentPattern {
                ref left,
                ref right,
            } => {
                left.traverse(visitor, ctx);
                right.traverse(visitor, ctx);
            }
        }
    }
}

impl<'ast> Visitable<'ast> for PropertyKey<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            PropertyKey::Computed(ref expression) => expression.traverse(visitor, ctx),
            PropertyKey::Literal(_) | PropertyKey::Binary(_) => {}
        }
    }
}

impl<'ast> Visitable<'ast> for Property<'ast> {
    type Parent = Node<'ast, Self>;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        match *self {
            Property::Shorthand(ref ident) => visitor.on_reference_use(ident, ctx),
            Property::Literal { ref key, ref value } => {
                key.traverse(visitor, ctx);
                value.traverse(visitor, ctx);
            }
            Property::Method { ref key, ref value } => {
                key.traverse(visitor, ctx);
                value.traverse(visitor, ctx);
            },
            Property::Spread { ref argument } => {
                argument.traverse(visitor, ctx);
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
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        if let Some(ref visitable) = *self {
            visitable.traverse(visitor, ctx);
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
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        self.item.traverse(visitor, ctx);
    }
}

impl<'ast, T> Visitable<'ast> for NodeList<'ast, T>
where
    T: Visitable<'ast, Parent = Node<'ast, T>>,
{
    type Parent = NoParent;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        for item in self {
            item.traverse(visitor, ctx);
        }
    }
}

impl<'ast> Visitable<'ast> for ExpressionList<'ast> {
    type Parent = NoParent;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        for node in self.iter() {
            node.traverse(visitor, ctx);
        }
    }
}

impl<'ast> Visitable<'ast> for StatementList<'ast> {
    type Parent = NoParent;

    #[inline]
    fn traverse<V>(&'ast self, visitor: &V, ctx: &mut V::Context)
    where
        V: Visitor<'ast>,
    {
        visitor.on_statement_list(*self, ctx);
        for node in self.iter() {
            node.traverse(visitor, ctx);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ratel::parse;
    use ScopeKind::*;

    struct TestContext<'ast> {
        depth: i32,
        max_depth: i32,
        scopes: Vec<ScopeKind>,
        used_vars: Vec<(&'ast str, i32)>,
        declared_vars: Vec<(&'ast str, i32)>,
    }

    impl<'ast> TestContext<'ast> {
        fn new() -> Self {
            TestContext {
                depth: 0,
                max_depth: 0,
                scopes: Vec::new(),
                used_vars: Vec::new(),
                declared_vars: Vec::new(),
            }
        }
    }

    struct ScopeTest;

    impl<'ast> StaticVisitor<'ast> for ScopeTest {
        type Context = TestContext<'ast>;

        fn on_enter_scope(kind: ScopeKind, ctx: &mut TestContext<'ast>) {
            ctx.scopes.push(kind);
            ctx.depth += 1;
            ctx.max_depth = ctx.max_depth.max(ctx.depth);
        }

        fn on_leave_scope(ctx: &mut TestContext<'ast>) {
            ctx.depth -= 1;
        }

        fn on_reference_use(ident: &Identifier<'ast>, ctx: &mut TestContext<'ast>) {
            ctx.used_vars.push((*ident, ctx.depth));
        }

        fn on_reference_declaration(ident: &Identifier<'ast>, ctx: &mut TestContext<'ast>) {
            ctx.declared_vars.push((*ident, ctx.depth));
        }

        fn register(_dv: &mut DynamicVisitor<'ast, TestContext<'ast>>) {
            unimplemented!()
        }
    }

    #[test]
    fn keeps_track_of_blocks() {
        let module = parse("{{{}}}").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[Block, Block, Block]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 3);
        assert_eq!(ctx.used_vars, &[]);
        assert_eq!(ctx.declared_vars, &[]);
    }

    #[test]
    fn keeps_track_of_declarations() {
        let module = parse("let foo; const bar = 42, doge;").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 0);
        assert_eq!(ctx.used_vars, &[]);
        assert_eq!(ctx.declared_vars, &[("foo", 0), ("bar", 0), ("doge", 0)]);
    }

    #[test]
    fn keeps_track_of_declarations_at_the_correct_depth() {
        let module = parse("let foo; { let foo; { let foo; }}").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[Block, Block]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 2);
        assert_eq!(ctx.used_vars, &[]);
        assert_eq!(ctx.declared_vars, &[("foo", 0), ("foo", 1), ("foo", 2)]);
    }

    #[test]
    fn keeps_track_of_uses() {
        let module = parse("doge = to + the + moon").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 0);
        assert_eq!(
            ctx.used_vars,
            &[("doge", 0), ("to", 0), ("the", 0), ("moon", 0)]
        );
        assert_eq!(ctx.declared_vars, &[]);
    }

    #[test]
    fn keeps_track_of_uses_at_the_correct_depth() {
        let module = parse("doge; { to; { the; { moon; }}}").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[Block, Block, Block]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 3);
        assert_eq!(
            ctx.used_vars,
            &[("doge", 0), ("to", 1), ("the", 2), ("moon", 3)]
        );
        assert_eq!(ctx.declared_vars, &[]);
    }

    #[test]
    fn function_and_class_are_declarations() {
        let module = parse("function foo() {} class Bar {}").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[Function]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 1);
        assert_eq!(ctx.used_vars, &[]);
        assert_eq!(ctx.declared_vars, &[("foo", 0), ("Bar", 0)]);
    }

    #[test]
    fn function_and_class_expressions_are_not_declarations() {
        let module = parse("(function foo() {}); (class Bar {});").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[Function]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 1);
        assert_eq!(ctx.used_vars, &[]);
        assert_eq!(ctx.declared_vars, &[]);
    }

    #[test]
    fn empty_class_has_no_scope() {
        let module = parse("class Doge {}").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 0);
        assert_eq!(ctx.used_vars, &[]);
        assert_eq!(ctx.declared_vars, &[("Doge", 0)]);
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

        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[Function, Function]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 2);
        assert_eq!(ctx.used_vars, &[("foo", 1), ("bar", 2)]);
        assert_eq!(ctx.declared_vars, &[("doge", 0)]);
    }

    #[test]
    fn object_property_shorthand_is_a_use() {
        let module = parse("const doge = { to, the, moon };").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 0);
        assert_eq!(ctx.used_vars, &[("to", 0), ("the", 0), ("moon", 0)]);
        assert_eq!(ctx.declared_vars, &[("doge", 0)]);
    }

    #[test]
    fn function_params_are_declarations() {
        let module = parse("function doge(to, the) { const moon; }").unwrap();
        let mut ctx = TestContext::new();

        module.traverse(&ScopeTest, &mut ctx);

        assert_eq!(ctx.scopes, &[Function]);
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, 1);
        assert_eq!(ctx.used_vars, &[]);
        assert_eq!(
            ctx.declared_vars,
            &[("doge", 0), ("to", 1), ("the", 1), ("moon", 1)]
        );
    }

    struct ParentsTestContext<'ast> {
        count: u32,
        parents: Vec<ParentNode<'ast>>,
    }

    impl<'ast> ParentsTestContext<'ast> {
        fn new() -> ParentsTestContext<'ast> {
            ParentsTestContext {
                count: 0,
                parents: Vec::new(),
            }
        }
    }

    impl<'ast> ParentTrackingContext<'ast> for ParentsTestContext<'ast> {
        #[inline]
        fn push_parent(&mut self, node: ParentNode<'ast>) {
            self.parents.push(node);
        }

        #[inline]
        fn pop_parent(&mut self) -> Option<ParentNode<'ast>> {
            self.parents.pop()
        }

        #[inline]
        fn get_parent(&mut self) -> Option<ParentNode<'ast>> {
            self.parents.last().cloned()
        }
    }

    struct ParentsTest;

    impl<'ast> StaticVisitor<'ast> for ParentsTest {
        type Context = ParentsTestContext<'ast>;

        fn push_parent(node: ParentNode<'ast>, ctx: &mut ParentsTestContext<'ast>) {
            ctx.count += 1;
            ctx.parents.push(node);
        }

        fn pop_parent(ctx: &mut ParentsTestContext<'ast>) {
            assert_eq!(ctx.parents.pop().is_some(), true);
        }

        fn register(_dv: &mut DynamicVisitor<'ast, ParentsTestContext<'ast>>) {
            unimplemented!()
        }
    }

    #[test]
    fn should_track_1_parent_for_every_statement_or_expression_node() {
        let module = parse("{{{1+2}}}").unwrap();
        let mut ctx = ParentsTestContext::new();

        module.traverse(&ParentsTest, &mut ctx);

        // 3 BlockStatements + 1 ExpressionStatement + 1 BinaryExpression = 5 Parents tracked
        // The 2 Literals don't have children, therefore they aren't tracked
        assert_eq!(ctx.count, 5);
        assert_eq!(ctx.parents.len(), 0);
    }
}
