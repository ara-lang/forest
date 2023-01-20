use ara_parser::tree::definition::function::ConcreteConstructorDefinition;
use ara_parser::tree::definition::function::{ConcreteMethodDefinition, FunctionDefinition};
use ara_parser::tree::definition::r#type::TypeDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::expression::function::AnonymousFunctionExpression;
use ara_parser::tree::statement::r#return::ReturnStatement;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct ReturnStatementAnalyzer;

impl ReturnStatementAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for ReturnStatementAnalyzer {
    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];
        if let Some(statement) = downcast::<ReturnStatement>(node) {
            for parent in ancestry.iter().rev() {
                if let Some(anonymous_function) = downcast::<AnonymousFunctionExpression>(*parent) {
                    if let Some(issue) = return_from_never_function(
                        source,
                        statement,
                        anonymous_function,
                        "function",
                        "function@anonymous",
                        &anonymous_function.return_type.type_definition,
                    ) {
                        issues.push(issue);
                    } else if !matches!(
                        statement,
                        ReturnStatement::Explicit {
                            expression: None,
                            ..
                        }
                    ) {
                        if let Some(issue) = return_from_void_function(
                            source,
                            statement,
                            anonymous_function,
                            "function",
                            "function@anonymous",
                            &anonymous_function.return_type.type_definition,
                        ) {
                            issues.push(issue);
                        }
                    }

                    break;
                }

                if let Some(function) = downcast::<FunctionDefinition>(*parent) {
                    if let Some(issue) = return_from_never_function(
                        source,
                        statement,
                        function,
                        "function",
                        &function.name.to_string(),
                        &function.return_type.type_definition,
                    ) {
                        issues.push(issue);
                    } else if !matches!(
                        statement,
                        ReturnStatement::Explicit {
                            expression: None,
                            ..
                        }
                    ) {
                        if let Some(issue) = return_from_void_function(
                            source,
                            statement,
                            function,
                            "function",
                            &function.name.to_string(),
                            &function.return_type.type_definition,
                        ) {
                            issues.push(issue);
                        }
                    }

                    break;
                }

                if let Some(method) = downcast::<ConcreteMethodDefinition>(*parent) {
                    if let Some(issue) = return_from_never_function(
                        source,
                        statement,
                        method,
                        "method",
                        &method.name.to_string(),
                        &method.return_type.type_definition,
                    ) {
                        issues.push(issue);
                    } else if !matches!(
                        statement,
                        ReturnStatement::Explicit {
                            expression: None,
                            ..
                        }
                    ) {
                        if let Some(issue) = return_from_void_function(
                            source,
                            statement,
                            method,
                            "method",
                            &method.name.to_string(),
                            &method.return_type.type_definition,
                        ) {
                            issues.push(issue);
                        }
                    }

                    break;
                }

                if let Some(constructor) = downcast::<ConcreteConstructorDefinition>(*parent) {
                    let issue = Issue::error(
                        AnalyzerIssueCode::CannotReturnFromConstructor,
                        "cannot return a value from constructor",
                    )
                    .with_source(
                        source,
                        statement.initial_position(),
                        statement.final_position(),
                    )
                    .with_annotation(Annotation::secondary(
                        source,
                        constructor.initial_position(),
                        constructor.parameters.final_position(),
                    ));

                    issues.push(issue);
                }
            }
        }

        issues
    }
}

fn return_from_void_function(
    source: &str,
    statement: &dyn Node,
    function: &dyn Node,
    function_type: &str,
    function_name: &str,
    return_type: &TypeDefinition,
) -> Option<Issue> {
    if let TypeDefinition::Void(_) = return_type {
        Some(
            Issue::error(
                AnalyzerIssueCode::CannotReturnAValueFromVoidFunction,
                format!(
                    "cannot return a value from void {} `{}`",
                    function_type, function_name
                ),
            )
            .with_source(
                source,
                statement.initial_position(),
                statement.final_position(),
            )
            .with_annotation(Annotation::secondary(
                source,
                function.initial_position(),
                function.final_position(),
            ))
            .with_annotation(Annotation::primary(
                source,
                return_type.initial_position(),
                return_type.final_position(),
            )),
        )
    } else {
        None
    }
}

fn return_from_never_function(
    source: &str,
    statement: &dyn Node,
    function: &dyn Node,
    function_type: &str,
    function_name: &str,
    return_type: &TypeDefinition,
) -> Option<Issue> {
    if let TypeDefinition::Never(_) = return_type {
        Some(
            Issue::error(
                AnalyzerIssueCode::CannotReturnFromNeverFunction,
                format!(
                    "cannot return from never {} `{}`",
                    function_type, function_name
                ),
            )
            .with_source(
                source,
                statement.initial_position(),
                statement.final_position(),
            )
            .with_annotation(Annotation::secondary(
                source,
                function.initial_position(),
                function.final_position(),
            ))
            .with_annotation(Annotation::primary(
                source,
                return_type.initial_position(),
                return_type.final_position(),
            )),
        )
    } else {
        None
    }
}
