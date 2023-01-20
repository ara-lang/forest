use ara_parser::tree::downcast;
use ara_parser::tree::expression::operator::ExceptionOperationExpression;
use ara_parser::tree::expression::Expression;
use ara_parser::tree::statement::block::BlockStatement;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct UnreachableCode;

impl UnreachableCode {
    pub fn new() -> Self {
        Self
    }
}

impl UnreachableCode {
    fn analyze_expression(source: &str, expression: &Expression) -> Option<Annotation> {
        let mut found_unreachable = None;
        match expression {
            Expression::ExceptionOperation(
                expression @ ExceptionOperationExpression::Throw { .. },
            ) => {
                found_unreachable = Some(
                    Annotation::secondary(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_message(format!(
                        "any code following this {} is unreachable",
                        expression.get_description()
                    )),
                );
            }
            Expression::ExitConstruct(expression) => {
                found_unreachable = Some(
                    Annotation::secondary(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_message(format!(
                        "any code following this {} is unreachable",
                        expression.get_description()
                    )),
                );
            }
            Expression::Parenthesized(expression) => {
                found_unreachable = Self::analyze_expression(source, &expression.expression);
            }
            _ => {}
        }

        found_unreachable
    }

    fn analyze_statement(source: &str, statement: &Statement) -> Option<Annotation> {
        let mut found_unreachable = None;
        match statement {
            Statement::Break(statement) => {
                found_unreachable = Some(
                    Annotation::secondary(
                        source,
                        statement.initial_position(),
                        statement.final_position(),
                    )
                    .with_message(format!(
                        "any code following this {} is unreachable",
                        statement.get_description()
                    )),
                );
            }
            Statement::Continue(statement) => {
                found_unreachable = Some(
                    Annotation::secondary(
                        source,
                        statement.initial_position(),
                        statement.final_position(),
                    )
                    .with_message(format!(
                        "any code following this {} is unreachable",
                        statement.get_description()
                    )),
                );
            }
            Statement::Return(statement) => {
                found_unreachable = Some(
                    Annotation::secondary(
                        source,
                        statement.initial_position(),
                        statement.final_position(),
                    )
                    .with_message(format!(
                        "any code following this {} is unreachable",
                        statement.get_description()
                    )),
                );
            }
            Statement::Expression(expression) => {
                found_unreachable = Self::analyze_expression(source, &expression.expression);
            }
            Statement::Block(block) => {
                for statement in block.statements.iter() {
                    if let Some(unreachable) = Self::analyze_statement(source, statement) {
                        found_unreachable = Some(unreachable);

                        break;
                    }
                }
            }
            Statement::Using(statement) => {
                for statement in statement.block.statements.iter() {
                    if let Some(unreachable) = Self::analyze_statement(source, statement) {
                        found_unreachable = Some(unreachable);

                        break;
                    }
                }
            }
            _ => {}
        }

        found_unreachable
    }
}

impl Visitor for UnreachableCode {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        if let Some(block) = downcast::<BlockStatement>(node) {
            let mut found_unreachable = None;
            for statement in &block.statements {
                if let Some(exit_annotation) = found_unreachable {
                    let issue = Issue::error(
                        AnalyzerIssueCode::UnreachableCode,
                        "unreachable code detected",
                    )
                    .with_source(
                        source,
                        statement.initial_position(),
                        block.statements.last().unwrap().final_position(),
                    )
                    .with_annotation(exit_annotation);

                    return vec![issue];
                }

                if let Some(statement) = downcast::<Statement>(statement) {
                    found_unreachable = Self::analyze_statement(source, statement);
                }
            }
        }

        vec![]
    }
}
