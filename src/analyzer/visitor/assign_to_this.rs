use ara_parser::tree::definition::function::{
    ConcreteConstructorDefinition, ConcreteMethodDefinition,
};
use ara_parser::tree::downcast;
use ara_parser::tree::expression::operator::AssignmentOperationExpression;
use ara_parser::tree::expression::Expression;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct AssignToThis;

impl AssignToThis {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for AssignToThis {
    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &[&dyn Node]) -> Vec<Issue> {
        if let Some(expression) = downcast::<AssignmentOperationExpression>(node) {
            if is_left_this(expression) {
                let issue = Issue::error(
                    AnalyzerIssueCode::CannotAssignToThis,
                    "cannot assign to $this",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                );

                for parent in ancestry {
                    if let Some(method) = downcast::<ConcreteMethodDefinition>(*parent) {
                        return vec![issue.with_annotation(Annotation::secondary(
                            source,
                            method.initial_position(),
                            method.return_type.final_position(),
                        ))];
                    }

                    if let Some(constructor) = downcast::<ConcreteConstructorDefinition>(*parent) {
                        return vec![issue.with_annotation(Annotation::secondary(
                            source,
                            constructor.initial_position(),
                            constructor.parameters.final_position(),
                        ))];
                    }
                }
            }
        }

        vec![]
    }
}

fn is_left_this(expression: &AssignmentOperationExpression) -> bool {
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
        | AssignmentOperationExpression::Coalesce { left, .. } => left,
    };

    if let Expression::Variable(variable) = left_expression.as_ref() {
        let name = variable.name.to_string();
        let lowercase_name = name.to_lowercase();

        if lowercase_name == "$this" {
            return true;
        }
    }

    false
}
