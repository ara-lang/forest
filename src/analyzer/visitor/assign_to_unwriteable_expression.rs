use ara_parser::tree::downcast;
use ara_parser::tree::expression::operator::AssignmentOperationExpression;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct AssignToUnwriteableExpression;

impl AssignToUnwriteableExpression {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for AssignToUnwriteableExpression {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        if let Some(expression) = downcast::<AssignmentOperationExpression>(node) {
            if is_left_unwriteable(expression) {
                let issue = Issue::error(
                    AnalyzerIssueCode::CannotAssignToUnwriteableExpression,
                    "cannot assign to an unwriteable expression",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                );

                return vec![issue];
            }
        }

        vec![]
    }
}

fn is_left_unwriteable(expression: &AssignmentOperationExpression) -> bool {
    let left_expression = match expression {
        AssignmentOperationExpression::Assignment { left, .. }
        | AssignmentOperationExpression::Addition { left, .. }
        | AssignmentOperationExpression::Subtraction { left, .. }
        | AssignmentOperationExpression::Multiplication { left, .. }
        | AssignmentOperationExpression::Division { left, .. }
        | AssignmentOperationExpression::Modulo { left, .. }
        | AssignmentOperationExpression::Exponentiation { left, .. }
        | AssignmentOperationExpression::Concat { left, .. }
        | AssignmentOperationExpression::BitwiseAnd { left, .. }
        | AssignmentOperationExpression::BitwiseOr { left, .. }
        | AssignmentOperationExpression::BitwiseXor { left, .. }
        | AssignmentOperationExpression::LeftShift { left, .. }
        | AssignmentOperationExpression::RightShift { left, .. }
        | AssignmentOperationExpression::Coalesce { left, .. } => left.as_ref(),
    };

    !left_expression.is_writable()
}
