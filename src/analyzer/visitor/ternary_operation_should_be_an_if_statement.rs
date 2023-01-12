use ara_parser::tree::downcast;
use ara_parser::tree::expression::operator::TernaryOperationExpression;
use ara_parser::tree::expression::Expression;
use ara_parser::tree::statement::block::BlockStatement;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct TernaryOperationShouldBeAnIfStatement;

impl TernaryOperationShouldBeAnIfStatement {
    pub fn new() -> Self {
        Self
    }
}

impl TernaryOperationShouldBeAnIfStatement {
    fn analyze_expression(source: &str, expression: &Expression) -> Vec<Issue> {
        match &expression {
            Expression::Parenthesized(expression) => {
                Self::analyze_expression(source, &expression.expression)
            }
            Expression::TernaryOperation(operation) => match operation {
                TernaryOperationExpression::Ternary { .. } => {
                    vec![Issue::error(
                        AnalyzerIssueCode::TernaryOperationShouldBeAnIfElseStatement,
                        "ternary operation should be an if-else statement",
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
                TernaryOperationExpression::ImplicitShortTernary { .. }
                | TernaryOperationExpression::ShortTernary { .. } => {
                    vec![Issue::error(
                        AnalyzerIssueCode::TernaryOperationShouldBeAnIfStatement,
                        "ternary operation should be an if statement",
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
            },
            _ => vec![],
        }
    }
}

impl Visitor for TernaryOperationShouldBeAnIfStatement {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(block) = downcast::<BlockStatement>(node) {
            for statement in &block.statements {
                if let Statement::Expression(expression) = statement {
                    for issue in Self::analyze_expression(source, &expression.expression) {
                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }
}
