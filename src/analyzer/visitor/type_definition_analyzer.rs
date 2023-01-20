use ara_parser::tree::definition::function::ConstructorParameterDefinition;
use ara_parser::tree::definition::function::FunctionLikeParameterDefinition;
use ara_parser::tree::definition::property::PropertyDefinition;
use ara_parser::tree::definition::r#type::TypeDefinition;
use ara_parser::tree::definition::template::TypeTemplateGroupDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct TypeDefinitionAnalyzer;

impl TypeDefinitionAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for TypeDefinitionAnalyzer {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(type_definition) = downcast::<TypeDefinition>(node) {
            if let TypeDefinition::Vec(keyword, templates) = type_definition {
                if let Some(issue) =
                    get_invalid_type_templates_count_issue(1, source, keyword, templates)
                {
                    return vec![issue];
                }
            }

            if let TypeDefinition::Dict(keyword, templates) = type_definition {
                if let Some(issue) =
                    get_invalid_type_templates_count_issue(2, source, keyword, templates)
                {
                    return vec![issue];
                }
            }

            if let TypeDefinition::Iterable(keyword, templates) = type_definition {
                if let Some(issue) =
                    get_invalid_type_templates_count_issue(2, source, keyword, templates)
                {
                    return vec![issue];
                }
            }

            if let TypeDefinition::Class(keyword, templates) = type_definition {
                if let Some(issue) =
                    get_invalid_type_templates_count_issue(1, source, keyword, templates)
                {
                    return vec![issue];
                }
            }

            if let TypeDefinition::Interface(keyword, templates) = type_definition {
                if let Some(issue) =
                    get_invalid_type_templates_count_issue(1, source, keyword, templates)
                {
                    return vec![issue];
                }
            }

            if let tuple @ TypeDefinition::Tuple {
                type_definitions, ..
            } = type_definition
            {
                for type_definition in &type_definitions.inner {
                    if type_definition.is_bottom() {
                        issues.push(
                            Issue::error(
                                AnalyzerIssueCode::BottomTypeCannotBeUsedInTuple,
                                format!(
                                    "bottom type `{{{}}}` cannot be used in a tuple",
                                    type_definition
                                ),
                            )
                            .with_source(
                                source,
                                type_definition.initial_position(),
                                type_definition.final_position(),
                            )
                            .with_annotation(Annotation::secondary(
                                source,
                                tuple.initial_position(),
                                tuple.final_position(),
                            )),
                        );
                    }
                }
            }

            if let union @ TypeDefinition::Union(inner_types) = type_definition {
                for type_definition in inner_types {
                    if type_definition.is_standalone() {
                        issues.push(
                            Issue::error(
                                AnalyzerIssueCode::StandaloneTypeCannotBeUsedInUnion,
                                format!(
                                    "standalone type `{{{}}}` cannot be used in a union",
                                    type_definition
                                ),
                            )
                            .with_source(
                                source,
                                type_definition.initial_position(),
                                type_definition.final_position(),
                            )
                            .with_annotation(Annotation::secondary(
                                source,
                                union.initial_position(),
                                union.final_position(),
                            )),
                        );
                    }
                }
            }

            if let intersection @ TypeDefinition::Intersection(inner_types) = type_definition {
                for type_definition in inner_types {
                    if type_definition.is_standalone() {
                        issues.push(
                            Issue::error(
                                AnalyzerIssueCode::StandaloneTypeCannotBeUsedInIntersection,
                                format!(
                                    "standalone type `{{{}}}` cannot be used in an intersection",
                                    type_definition
                                ),
                            )
                            .with_source(
                                source,
                                type_definition.initial_position(),
                                type_definition.final_position(),
                            )
                            .with_annotation(Annotation::secondary(
                                source,
                                intersection.initial_position(),
                                intersection.final_position(),
                            )),
                        );
                    } else if type_definition.is_scalar() {
                        issues.push(
                            Issue::error(
                                AnalyzerIssueCode::ScalarTypeCannotBeUsedInIntersection,
                                format!(
                                    "scalar type `{{{}}}` cannot be used in an intersection",
                                    type_definition
                                ),
                            )
                            .with_source(
                                source,
                                type_definition.initial_position(),
                                type_definition.final_position(),
                            )
                            .with_annotation(Annotation::secondary(
                                source,
                                intersection.initial_position(),
                                intersection.final_position(),
                            )),
                        );
                    }
                }
            }

            if let nullable @ TypeDefinition::Nullable(_, type_definition) = type_definition {
                if type_definition.is_standalone() {
                    issues.push(
                        Issue::error(
                            AnalyzerIssueCode::StandaloneTypeCannotBeNullable,
                            format!(
                                "standalone type `{{{}}}` cannot be nullable",
                                type_definition
                            ),
                        )
                        .with_source(
                            source,
                            type_definition.initial_position(),
                            type_definition.final_position(),
                        )
                        .with_annotation(Annotation::secondary(
                            source,
                            nullable.initial_position(),
                            nullable.final_position(),
                        )),
                    );
                }
            }
        }

        if let Some(parameter) = downcast::<FunctionLikeParameterDefinition>(node) {
            if parameter.type_definition.is_bottom() {
                issues.push(
                    Issue::error(
                        AnalyzerIssueCode::BottomTypeCannotBeUsedForParameter,
                        format!(
                            "bottom type `{{{}}}` cannot be used for a parameter",
                            parameter.type_definition,
                        ),
                    )
                    .with_source(
                        source,
                        parameter.type_definition.initial_position(),
                        parameter.type_definition.final_position(),
                    )
                    .with_annotation(Annotation::secondary(
                        source,
                        parameter.initial_position(),
                        parameter.final_position(),
                    )),
                );
            }
        }

        if let Some(parameter) = downcast::<ConstructorParameterDefinition>(node) {
            if parameter.type_definition.is_bottom() {
                issues.push(
                    Issue::error(
                        AnalyzerIssueCode::BottomTypeCannotBeUsedForParameter,
                        format!(
                            "bottom type `{{{}}}` cannot be used for a parameter",
                            parameter.type_definition,
                        ),
                    )
                    .with_source(
                        source,
                        parameter.type_definition.initial_position(),
                        parameter.type_definition.final_position(),
                    )
                    .with_annotation(Annotation::secondary(
                        source,
                        parameter.initial_position(),
                        parameter.final_position(),
                    )),
                );
            }
        }

        if let Some(property) = downcast::<PropertyDefinition>(node) {
            if property.type_definition.is_bottom() {
                issues.push(
                    Issue::error(
                        AnalyzerIssueCode::BottomTypeCannotBeUsedForProperty,
                        format!(
                            "bottom type `{{{}}}` cannot be used for a property",
                            property.type_definition,
                        ),
                    )
                    .with_source(
                        source,
                        property.type_definition.initial_position(),
                        property.type_definition.final_position(),
                    )
                    .with_annotation(Annotation::secondary(
                        source,
                        property.initial_position(),
                        property.final_position(),
                    )),
                );
            }
        }

        issues
    }
}

fn get_invalid_type_templates_count_issue(
    expected: usize,
    source: &str,
    node: &dyn Node,
    templates: &TypeTemplateGroupDefinition,
) -> Option<Issue> {
    let len = templates.members.inner.len();
    if len != expected {
        let mut issue = Issue::error(
            AnalyzerIssueCode::InvalidGenericArgumentsCount,
            format!(
                "this type takes exactly {} generic argument(s) but {} generic argument(s) were supplied",
                expected,
                len
            ),
                ).with_source(
            source,
            node.initial_position(),
            node.final_position(),
        );

        if len > expected {
            for template in &templates.members.inner[expected..] {
                issue = issue.with_annotation(
                    Annotation::secondary(
                        source,
                        template.initial_position(),
                        template.final_position(),
                    )
                    .with_message("help: remove this generic argument"),
                );
            }
        }

        Some(issue)
    } else {
        None
    }
}
