use std::collections::hash_map::Entry;

use rustc_hash::FxHashMap;

use ara_parser::tree::definition::function::ConstructorParameterListDefinition;
use ara_parser::tree::definition::function::FunctionLikeParameterListDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct DuplicateParameter;

impl DuplicateParameter {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for DuplicateParameter {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(parameter_list) = downcast::<FunctionLikeParameterListDefinition>(node) {
            let mut map = FxHashMap::default();

            for parameter in &parameter_list.parameters.inner {
                let name = parameter.variable.name.to_string();
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
            }
        }

        if let Some(parameter_list) = downcast::<ConstructorParameterListDefinition>(node) {
            let mut map = FxHashMap::default();

            for parameter in &parameter_list.parameters.inner {
                let name = parameter.variable.name.to_string();
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
            }
        }

        issues
    }
}

fn duplicate_parameter(
    source: &str,
    name: &str,
    previous: &(usize, usize),
    parameter: &dyn Node,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::NoDuplicateParameter,
        format!("the parameter `{}` is defined multiple times", name),
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
