use ara_parser::tree::definition::function::ConstructorParameterListDefinition;
use ara_parser::tree::definition::function::FunctionLikeParameterListDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::variable::Variable;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct DefaultForVariadic;

impl DefaultForVariadic {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for DefaultForVariadic {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];
        if let Some(parameter_list) = downcast::<FunctionLikeParameterListDefinition>(node) {
            for parameter in &parameter_list.parameters.inner {
                if let (Some(_), Some(default)) = (&parameter.ellipsis, &parameter.default) {
                    issues.push(variadic_parameter_cannot_be_optional(
                        source,
                        parameter,
                        &parameter.variable,
                        default,
                    ));
                }
            }
        }

        if let Some(parameter_list) = downcast::<ConstructorParameterListDefinition>(node) {
            for parameter in &parameter_list.parameters.inner {
                if let (Some(_), Some(default)) = (&parameter.ellipsis, &parameter.default) {
                    issues.push(variadic_parameter_cannot_be_optional(
                        source,
                        parameter,
                        &parameter.variable,
                        default,
                    ));
                }
            }
        }

        issues
    }
}

fn variadic_parameter_cannot_be_optional(
    source: &str,
    parameter: &dyn Node,
    variable: &Variable,
    default: &dyn Node,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::VariadicParameterCannotBeOptional,
        format!(
            "parameter `{}` cannot be optional because it is variadic",
            variable
        ),
    )
    .with_source(source, default.initial_position(), default.final_position())
    .with_annotation(Annotation::secondary(
        source,
        parameter.initial_position(),
        variable.final_position(),
    ))
    .with_note("help: remove the default value")
}
