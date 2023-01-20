use ara_parser::tree::downcast;
use ara_parser::tree::expression::Expression;
use ara_parser::tree::statement::block::BlockStatement;
use ara_parser::tree::statement::control_flow::IfElseBlockStatement;
use ara_parser::tree::statement::control_flow::IfStatement;
use ara_parser::tree::statement::r#try::TryStatement;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct TryStatementAnalyzer;

impl TryStatementAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for TryStatementAnalyzer {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];
        if let Some(statement) = downcast::<TryStatement>(node) {
            match &statement.finally {
                Some(statement) => {
                    if let Some(unsafe_node) = find_unsafe_node(&statement.block) {
                        issues.push(
                            Issue::warning(
                                AnalyzerIssueCode::UnsafeFinallyBlock,
                                "unsafe code in finally block",
                            )
                            .with_source(
                                source,
                                unsafe_node.initial_position(),
                                unsafe_node.final_position(),
                            )
                            .with_annotation(Annotation::secondary(
                                source,
                                statement.initial_position(),
                                statement.final_position(),
                            )),
                        );
                    }
                }
                None => {
                    if statement.catches.is_empty() {
                        issues.push(
                            Issue::error(
                                AnalyzerIssueCode::TryBlockMustBeFollowedByCatchOrFinally,
                                "try block must be followed by catch or finally block",
                            )
                            .with_source(
                                source,
                                statement.initial_position(),
                                statement.final_position(),
                            ),
                        );
                    }
                }
            }
        }

        issues
    }
}

fn find_unsafe_node(block: &BlockStatement) -> Option<&dyn Node> {
    for statement in &block.statements {
        let unsafe_node = match &statement {
            Statement::DoWhile(statement) => find_unsafe_node(&statement.block),
            Statement::While(statement) => find_unsafe_node(&statement.block),
            Statement::For(statement) => find_unsafe_node(&statement.block),
            Statement::Foreach(statement) => find_unsafe_node(&statement.block),
            Statement::Break(statement) => Some(statement.as_ref() as &dyn Node),
            Statement::Continue(statement) => Some(statement.as_ref() as &dyn Node),
            Statement::If(statement) => find_unsafe_node_in_if(statement),
            Statement::Using(statement) => find_unsafe_node(&statement.block),
            Statement::Try(statement) => {
                if let Some(unsafe_node) = find_unsafe_node(&statement.block) {
                    return Some(unsafe_node);
                }

                for catch in &statement.catches {
                    if let Some(unsafe_node) = find_unsafe_node(&catch.block) {
                        return Some(unsafe_node);
                    }
                }

                if let Some(finally) = &statement.finally {
                    if let Some(unsafe_node) = find_unsafe_node(&finally.block) {
                        return Some(unsafe_node);
                    }
                }

                None
            }
            Statement::Expression(expression) => match &expression.expression {
                Expression::ExitConstruct(construct) => Some(construct as &dyn Node),
                Expression::ExceptionOperation(operation) => Some(operation as &dyn Node),
                Expression::GeneratorOperation(operation) => Some(operation as &dyn Node),
                _ => None,
            },
            Statement::Return(statement) => Some(statement.as_ref() as &dyn Node),
            Statement::Block(block) => find_unsafe_node(block),
            Statement::Empty(_) => None,
        };

        if let Some(unsafe_node) = unsafe_node {
            return Some(unsafe_node);
        }
    }

    None
}

fn find_unsafe_node_in_if(statement: &IfStatement) -> Option<&dyn Node> {
    if let Some(unsafe_node) = find_unsafe_node(&statement.block) {
        return Some(unsafe_node);
    }

    for elseif in &statement.elseifs {
        if let Some(unsafe_node) = find_unsafe_node(&elseif.block) {
            return Some(unsafe_node);
        }
    }

    if let Some(r#else) = &statement.r#else {
        match &r#else.block {
            IfElseBlockStatement::If(if_statement) => {
                if let Some(unsafe_node) = find_unsafe_node_in_if(if_statement) {
                    return Some(unsafe_node);
                }
            }
            IfElseBlockStatement::Block(block) => {
                if let Some(unsafe_node) = find_unsafe_node(block) {
                    return Some(unsafe_node);
                }
            }
        }
    }

    None
}
