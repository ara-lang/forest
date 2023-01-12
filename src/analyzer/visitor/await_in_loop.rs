use ara_parser::tree::downcast;
use ara_parser::tree::expression::function::AnonymousFunctionExpression;
use ara_parser::tree::expression::function::ArrowFunctionExpression;
use ara_parser::tree::expression::operator::AsyncOperationExpression;
use ara_parser::tree::expression::operator::ExceptionOperationExpression;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct AwaitInLoop;

impl AwaitInLoop {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for AwaitInLoop {
    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        if let Some(expression @ AsyncOperationExpression::Await { .. }) =
            downcast::<AsyncOperationExpression>(node)
        {
            for ancestor in ancestry.iter().rev() {
                if let Some(_) = downcast::<AnonymousFunctionExpression>(*ancestor) {
                    // Allow await in anonymous functions
                    break;
                }

                if let Some(_) = downcast::<ArrowFunctionExpression>(*ancestor) {
                    // Allow await in arrow functions
                    break;
                }

                if let Some(ExceptionOperationExpression::Throw { .. }) =
                    downcast::<ExceptionOperationExpression>(*ancestor)
                {
                    // Allow await in throw expressions
                    break;
                }

                if let Some(statement) = downcast::<Statement>(*ancestor) {
                    match statement {
                        Statement::DoWhile(_)
                        | Statement::While(_)
                        | Statement::For(_)
                        | Statement::Foreach(_) => {
                            let issue = Issue::note(
                                AnalyzerIssueCode::DontAwaitInLoop,
                                "awaiting in a loop is not recommended",
                                source,
                                expression.initial_position(),
                                expression.final_position(),
                            )
                            .with_annotation(Annotation::secondary(
                                source,
                                statement.initial_position(),
                                statement.final_position(),
                            ));

                            return vec![issue];
                        }
                        Statement::Return(_) => {
                            // Allow await in return statements
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        vec![]
    }
}
