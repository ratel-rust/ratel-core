use toolshed::list::ListBuilder;
use ast::{NodeList, Node, ExpressionNode};
use ast::expression::{TemplateLiteral, TaggedTemplateExpression};
use parser::expression::ExpressionHandler;
use parser::{Parser, ANY};
use lexer::Token::*;

pub struct TemplateStringLiteralHandler;
pub struct TemplateExpressionHandler;

impl ExpressionHandler for TemplateStringLiteralHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        let quasi = par.lexer.quasi;
        let quasi = par.node_consume(quasi);

        par.node_at(quasi.start, quasi.end, TemplateLiteral {
            expressions: NodeList::empty(),
            quasis: NodeList::from(par.arena, quasi)
        })
    }
}

impl ExpressionHandler for TemplateExpressionHandler {
    fn expression<'ast>(par: &mut Parser<'ast>) -> ExpressionNode<'ast> {
        par.template_literal()
    }
}

impl<'ast> Parser<'ast> {
    pub fn template_string<T>(&mut self) -> Node<'ast, T>
    where
        T: Copy + From<TemplateLiteral<'ast>>,
    {
        let quasi = self.lexer.quasi;
        let quasi = self.node_consume(quasi);

        self.node_at(quasi.start, quasi.end, TemplateLiteral {
            expressions: NodeList::empty(),
            quasis: NodeList::from(self.arena, quasi)
        })
    }

    pub fn template_literal<T>(&mut self) -> Node<'ast, T>
    where
        T: Copy + From<TemplateLiteral<'ast>>,
    {
        let quasi = self.lexer.quasi;
        let quasi = self.node_consume(quasi);
        let start = quasi.start;
        let end;

        let expression = self.expression::<ANY>();

        match self.lexer.token {
            BraceClose => self.lexer.read_template_kind(),
            _          => self.error(),
        }

        let quasis = ListBuilder::new(self.arena, quasi);
        let expressions = ListBuilder::new(self.arena, expression);

        loop {
            match self.lexer.token {
                TemplateOpen => {
                    let quasi = self.lexer.quasi;
                    quasis.push(self.arena, self.node_consume(quasi));
                    expressions.push(self.arena, self.expression::<ANY>());

                    match self.lexer.token {
                        BraceClose => self.lexer.read_template_kind(),
                        _          => {
                            end = self.lexer.end();
                            self.error::<()>();
                            break;
                        }
                    }
                },
                TemplateClosed => {
                    let quasi = self.lexer.quasi;
                    end = self.lexer.end();
                    quasis.push(self.arena, self.node_consume(quasi));
                    break;
                },
                _ => {
                    end = self.lexer.end();
                    self.error::<()>();
                    break;
                }
            }
        }

        self.node_at(start, end, TemplateLiteral {
            expressions: expressions.as_list(),
            quasis: quasis.as_list(),
        })
    }

    pub fn tagged_template_expression(&mut self, tag: ExpressionNode<'ast>) -> ExpressionNode<'ast> {
        let quasi = self.template_literal();

        self.node_at(tag.start, quasi.end, TaggedTemplateExpression {
            tag,
            quasi,
        })
    }
}
