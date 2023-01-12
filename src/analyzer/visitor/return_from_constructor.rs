use ara_parser::tree::definition::function::ConcreteConstructorDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::expression::function::AnonymousFunctionExpression;
use ara_parser::tree::statement::r#return::ReturnStatement;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct ReturnFromConstructor;

impl ReturnFromConstructor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for ReturnFromConstructor {
    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        if let Some(statement) = downcast::<ReturnStatement>(node) {
            for parent in ancestry.iter().rev() {
                if let Some(_) = downcast::<AnonymousFunctionExpression>(*parent) {
                    break;
                }

                if let Some(constructor) = downcast::<ConcreteConstructorDefinition>(*parent) {
                    let issue = Issue::error(
                        AnalyzerIssueCode::CannotReturnFromConstructor,
                        "cannot return a value from constructor",
                        source,
                        statement.initial_position(),
                        statement.final_position(),
                    )
                    .with_annotation(Annotation::secondary(
                        source,
                        constructor.initial_position(),
                        constructor.parameters.final_position(),
                    ));

                    return vec![issue];
                }
            }
        }

        vec![]
    }
}
