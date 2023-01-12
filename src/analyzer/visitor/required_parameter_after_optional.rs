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

pub struct RequiredParameterAfterOptional;

impl RequiredParameterAfterOptional {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for RequiredParameterAfterOptional {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(parameter_list) = downcast::<FunctionLikeParameterListDefinition>(node) {
            let mut optional_parameter: Option<&FunctionLikeParameterDefinition> = None;

            for parameter in &parameter_list.parameters.inner {
                if let Some(previous_parameter) = optional_parameter {
                    if parameter.default.is_none() {
                        let issue = get_required_parameter_issue(
                            source,
                            previous_parameter,
                            &previous_parameter.variable.name,
                            parameter,
                            &parameter.variable.name,
                        );

                        issues.push(issue);
                    }
                } else if parameter.default.is_some() {
                    optional_parameter = Some(parameter);
                }
            }
        }

        if let Some(parameter_list) = downcast::<ConstructorParameterListDefinition>(node) {
            let mut optional_parameter: Option<&ConstructorParameterDefinition> = None;

            for parameter in &parameter_list.parameters.inner {
                if let Some(previous_parameter) = optional_parameter {
                    if parameter.default.is_none() {
                        let issue = get_required_parameter_issue(
                            source,
                            previous_parameter,
                            &previous_parameter.variable.name,
                            parameter,
                            &parameter.variable.name,
                        );

                        issues.push(issue);
                    }
                } else if parameter.default.is_some() {
                    optional_parameter = Some(parameter);
                }
            }
        }

        issues
    }
}

fn get_required_parameter_issue(
    source: &str,
    previous: &dyn Node,
    previous_name: &ByteString,
    parameter: &dyn Node,
    parameter_name: &ByteString,
) -> Issue {
    let issue = Issue::error(
        AnalyzerIssueCode::NoRequiredParameterAfterOptional,
        format!(
            "required parameter `{}` declared after optional parameter `{}`",
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
        .with_message("optional parameter"),
    )
    .with_note(format!(
        "move the optional parameter `{}` after the required parameter `{}`",
        previous_name, parameter_name,
    ));

    issue
}
