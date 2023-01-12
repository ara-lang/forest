use ara_parser::lexer::byte_string::ByteString;
use ara_parser::tree::definition::function::ConstructorParameterDefinition;
use ara_parser::tree::definition::function::ConstructorParameterListDefinition;
use ara_parser::tree::definition::function::FunctionLikeParameterDefinition;
use ara_parser::tree::definition::function::FunctionLikeParameterListDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct ParametersAfterVariadic;

impl ParametersAfterVariadic {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for ParametersAfterVariadic {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        let mut issues = vec![];
        if let Some(parameter_list) = downcast::<FunctionLikeParameterListDefinition>(node) {
            let mut variadic_parameter: Option<&FunctionLikeParameterDefinition> = None;

            for parameter in &parameter_list.parameters.inner {
                if let Some(previous_parameter) = variadic_parameter {
                    let issue = get_additional_parameter_issue(
                        source,
                        previous_parameter,
                        &previous_parameter.variable.name,
                        parameter,
                        &parameter.variable.name,
                        parameter.ellipsis.is_some(),
                    );

                    issues.push(issue);
                } else if parameter.ellipsis.is_some() {
                    variadic_parameter = Some(parameter);
                }
            }
        } else if let Some(parameter_list) = downcast::<ConstructorParameterListDefinition>(node) {
            let mut variadic_parameter: Option<&ConstructorParameterDefinition> = None;

            let mut issues = vec![];
            for parameter in &parameter_list.parameters.inner {
                if let Some(previous_parameter) = variadic_parameter {
                    let issue = get_additional_parameter_issue(
                        source,
                        previous_parameter,
                        &previous_parameter.variable.name,
                        parameter,
                        &parameter.variable.name,
                        parameter.ellipsis.is_some(),
                    );

                    issues.push(issue);
                } else if parameter.ellipsis.is_some() {
                    variadic_parameter = Some(parameter);
                }
            }
        }

        issues
    }
}

fn get_additional_parameter_issue(
    source: &str,
    previous: &dyn Node,
    previous_name: &ByteString,
    parameter: &dyn Node,
    parameter_name: &ByteString,
    is_variadic: bool,
) -> Issue {
    let mut issue = Issue::error(
        AnalyzerIssueCode::NoMoreParametersAfterVariadic,
        format!(
            "parameter `{}` cannot appear after variadic parameter `{}`",
            parameter_name, previous_name,
        ),
        source,
        parameter.initial_position(),
        parameter.final_position(),
    )
    .with_annotation(
        Annotation::secondary(
            source,
            previous.initial_position(),
            previous.final_position(),
        )
        .with_message("variadic parameter"),
    );

    if is_variadic {
        issue = issue.with_note(format!("remove the variadic parameter `{}`", previous_name,));
    } else {
        issue = issue.with_note(format!(
            "move the variadic parameter `{}` after the parameter `{}`",
            previous_name, parameter_name,
        ));
    }

    issue
}
