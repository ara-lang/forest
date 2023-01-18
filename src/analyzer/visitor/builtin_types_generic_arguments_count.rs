use ara_parser::tree::definition::r#type::TypeDefinition;
use ara_parser::tree::definition::template::TypeTemplateGroupDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct BuiltinTypesGenericArgumentsCount;

impl BuiltinTypesGenericArgumentsCount {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for BuiltinTypesGenericArgumentsCount {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
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
        }

        vec![]
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
