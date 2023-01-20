use std::collections::hash_map::Entry;

use ara_parser::tree::variable::Variable;
use rustc_hash::FxHashMap;

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

#[derive(Debug, Default)]
pub struct ParamteresAnalyzer;

impl ParamteresAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for ParamteresAnalyzer {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(parameter_list) = downcast::<FunctionLikeParameterListDefinition>(node) {
            let mut map = FxHashMap::default();
            let mut optional_parameter: Option<&FunctionLikeParameterDefinition> = None;
            let mut variadic_parameter: Option<&FunctionLikeParameterDefinition> = None;

            for parameter in &parameter_list.parameters.inner {
                let name = parameter.variable.name.to_string().to_lowercase();

                match map.entry(name) {
                    Entry::Occupied(entry) => {
                        issues.push(duplicate_parameter(
                            source,
                            entry.key(),
                            entry.get(),
                            parameter,
                        ));
                    }
                    Entry::Vacant(entry) => {
                        entry.insert((parameter.initial_position(), parameter.final_position()));
                    }
                }

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

                    if let Some(default) = &parameter.default {
                        issues.push(variadic_parameter_cannot_be_optional(
                            source,
                            parameter,
                            &parameter.variable,
                            default,
                        ));
                    }
                }
            }
        }

        if let Some(parameter_list) = downcast::<ConstructorParameterListDefinition>(node) {
            let mut map = FxHashMap::default();
            let mut optional_parameter: Option<&ConstructorParameterDefinition> = None;
            let mut variadic_parameter: Option<&ConstructorParameterDefinition> = None;

            for parameter in &parameter_list.parameters.inner {
                let name = parameter.variable.name.to_string().to_lowercase();

                match map.entry(name) {
                    Entry::Occupied(entry) => {
                        issues.push(duplicate_parameter(
                            source,
                            entry.key(),
                            entry.get(),
                            parameter,
                        ));
                    }
                    Entry::Vacant(entry) => {
                        entry.insert((parameter.initial_position(), parameter.final_position()));
                    }
                }

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

                    if let Some(default) = &parameter.default {
                        issues.push(variadic_parameter_cannot_be_optional(
                            source,
                            parameter,
                            &parameter.variable,
                            default,
                        ));
                    }
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
    Issue::error(
        AnalyzerIssueCode::NoRequiredParameterAfterOptional,
        format!(
            "required parameter `{}` declared after optional parameter `{}`",
            parameter_name, previous_name,
        ),
    )
    .with_source(
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
    ))
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
    )
    .with_source(
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
        issue = issue.with_note(format!("remove the variadic parameter `{}`", previous_name));
    } else {
        issue = issue.with_note(format!(
            "move the variadic parameter `{}` after the parameter `{}`",
            previous_name, parameter_name,
        ));
    }

    issue
}

fn duplicate_parameter(
    source: &str,
    name: &str,
    previous: &(usize, usize),
    parameter: &dyn Node,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::NoDuplicateParameter,
        format!("parameter `{}` is defined multiple times", name),
    )
    .with_source(
        source,
        parameter.initial_position(),
        parameter.final_position(),
    )
    .with_annotation(
        Annotation::secondary(source, previous.0, previous.1).with_message(format!(
            "previous definition of the parameter `{}` here",
            name
        )),
    )
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
