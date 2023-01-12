use std::collections::hash_map::Entry;

use ara_parser::tree::definition::namespace::NamespaceDefinition;
use ara_parser::tree::definition::r#use::UseDefinition;
use ara_parser::tree::identifier::Identifier;
use rustc_hash::FxHashMap;

use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct RedundantImport {
    last_source_namespace: FxHashMap<String, (String, usize, usize)>,
    used_functions: FxHashMap<String, FxHashMap<String, (usize, usize)>>,
    used_constants: FxHashMap<String, FxHashMap<String, (usize, usize)>>,
    used_classish: FxHashMap<String, FxHashMap<String, (usize, usize)>>,
    non_aliased_functions: FxHashMap<String, FxHashMap<String, (usize, usize)>>,
    non_aliased_constants: FxHashMap<String, FxHashMap<String, (usize, usize)>>,
    non_aliased_classish: FxHashMap<String, FxHashMap<String, (usize, usize)>>,
}

impl RedundantImport {
    pub fn new() -> Self {
        Self {
            last_source_namespace: FxHashMap::default(),
            used_functions: FxHashMap::default(),
            used_constants: FxHashMap::default(),
            used_classish: FxHashMap::default(),
            non_aliased_functions: FxHashMap::default(),
            non_aliased_constants: FxHashMap::default(),
            non_aliased_classish: FxHashMap::default(),
        }
    }
}

impl Visitor for RedundantImport {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(namespace) = downcast::<NamespaceDefinition>(node) {
            self.last_source_namespace.insert(
                source.to_string(),
                (
                    namespace.name.value.to_string(),
                    namespace.initial_position(),
                    namespace.final_position(),
                ),
            );
        }

        if let Some(use_definition) = downcast::<UseDefinition>(node) {
            let mut redundant_import = false;
            let mut redundant_alias = None;
            let mut duplicate_import = None;
            let mut duplicate_import_under_alias = None;

            match &use_definition {
                UseDefinition::Default { name, alias, .. } => {
                    let (actual_name, used_namespace) = get_name_and_namespace(&name);
                    let mut used_name = actual_name.to_string();

                    // handle `use Foo as Foo;`
                    if let Some(alias) = alias {
                        if alias.alias.value.to_string() == used_name {
                            redundant_alias = Some((name.final_position(), alias));
                        }

                        used_name = alias.alias.value.to_string();
                    } else if let Some(last_namespace) = self.last_source_namespace.get(source) {
                        if used_namespace == last_namespace.0 {
                            redundant_import = true;
                        }
                    }

                    match self
                        .used_classish
                        .entry(source.to_string())
                        .or_insert(FxHashMap::default())
                        .entry(used_name)
                    {
                        Entry::Occupied(previous) => {
                            duplicate_import = Some(previous.get().clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert((
                                use_definition.initial_position(),
                                use_definition.final_position(),
                            ));

                            match self
                                .non_aliased_classish
                                .entry(source.to_owned())
                                .or_insert(FxHashMap::default())
                                .entry(actual_name)
                            {
                                Entry::Occupied(previous) => {
                                    duplicate_import_under_alias = Some(previous.get().clone());
                                }
                                Entry::Vacant(entry) => {
                                    entry.insert((
                                        use_definition.initial_position(),
                                        use_definition.final_position(),
                                    ));
                                }
                            }
                        }
                    }
                }
                UseDefinition::Function { name, alias, .. } => {
                    let (actual_name, used_namespace) = get_name_and_namespace(&name);
                    let mut used_name = actual_name.to_string();

                    // handle `use function Foo as Foo;`
                    if let Some(alias) = alias {
                        if alias.alias.value.to_string() == used_name {
                            redundant_alias = Some((name.final_position(), alias));
                        }

                        used_name = alias.alias.value.to_string();
                    } else if let Some(last_namespace) = self.last_source_namespace.get(source) {
                        if used_namespace == last_namespace.0 {
                            redundant_import = true;
                        }
                    }

                    match self
                        .used_functions
                        .entry(source.to_string())
                        .or_insert(FxHashMap::default())
                        .entry(used_name)
                    {
                        Entry::Occupied(previous) => {
                            duplicate_import = Some(previous.get().clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert((
                                use_definition.initial_position(),
                                use_definition.final_position(),
                            ));

                            match self
                                .non_aliased_functions
                                .entry(source.to_owned())
                                .or_insert(FxHashMap::default())
                                .entry(actual_name)
                            {
                                Entry::Occupied(previous) => {
                                    duplicate_import_under_alias = Some(previous.get().clone());
                                }
                                Entry::Vacant(entry) => {
                                    entry.insert((
                                        use_definition.initial_position(),
                                        use_definition.final_position(),
                                    ));
                                }
                            }
                        }
                    }
                }
                UseDefinition::Constant { name, alias, .. } => {
                    let (actual_name, used_namespace) = get_name_and_namespace(&name);
                    let mut used_name = actual_name.to_string();

                    // handle `use const Foo as Foo;`
                    if let Some(alias) = alias {
                        if alias.alias.value.to_string() == used_name {
                            redundant_alias = Some((name.final_position(), alias));
                        }

                        used_name = alias.alias.value.to_string();
                    } else if let Some(last_namespace) = self.last_source_namespace.get(source) {
                        if used_namespace == last_namespace.0 {
                            redundant_import = true;
                        }
                    }

                    match self
                        .used_constants
                        .entry(source.to_string())
                        .or_insert(FxHashMap::default())
                        .entry(used_name)
                    {
                        Entry::Occupied(previous) => {
                            duplicate_import = Some(previous.get().clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert((
                                use_definition.initial_position(),
                                use_definition.final_position(),
                            ));

                            match self
                                .non_aliased_constants
                                .entry(source.to_owned())
                                .or_insert(FxHashMap::default())
                                .entry(actual_name)
                            {
                                Entry::Occupied(previous) => {
                                    duplicate_import_under_alias = Some(previous.get().clone());
                                }
                                Entry::Vacant(entry) => {
                                    entry.insert((
                                        use_definition.initial_position(),
                                        use_definition.final_position(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            if redundant_import {
                let mut issue = Issue::note(
                    AnalyzerIssueCode::RedundantUse,
                    format!("redudant use definition"),
                    source,
                    use_definition.initial_position(),
                    use_definition.final_position(),
                );

                if let Some(namespace) = self.last_source_namespace.get(source) {
                    issue = issue.with_annotation(
                        Annotation::secondary(source, namespace.1, namespace.2)
                            .with_message(format!("namespace `{}` defined here", namespace.0)),
                    );
                }

                issues.push(issue);
            }

            if let Some(previous_definition) = duplicate_import {
                let issue = Issue::error(
                    AnalyzerIssueCode::DuplicateUseDefinition,
                    format!("duplicate use definition"),
                    source,
                    use_definition.initial_position(),
                    use_definition.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source,
                    previous_definition.0,
                    previous_definition.1,
                ));

                issues.push(issue);
            }

            if let Some(previous_definition) = duplicate_import_under_alias {
                let issue = Issue::warning(
                    AnalyzerIssueCode::DuplicateUseDefinitionUnderAlias,
                    format!("duplicate use definition under alias"),
                    source,
                    use_definition.initial_position(),
                    use_definition.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source,
                    previous_definition.0,
                    previous_definition.1,
                ));

                issues.push(issue);
            }

            if let Some((last, redundant_alias)) = redundant_alias {
                let issue = Issue::note(
                    AnalyzerIssueCode::RedundantUseDefinitionAlias,
                    format!("redundant use definition alias"),
                    source,
                    redundant_alias.initial_position(),
                    redundant_alias.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source,
                    use_definition.initial_position(),
                    last,
                ));

                issues.push(issue);
            }
        }

        issues
    }
}

fn get_name_and_namespace(fqn: &Identifier) -> (String, String) {
    let used_name = fqn.value.to_string();
    let parts = used_name.split('\\').collect::<Vec<&str>>();
    let (used_namespace, used_short_name) = parts.split_at(parts.len() - 1);
    let used_namespace = used_namespace.join("\\");
    let used_short_name = used_short_name[0];

    (used_short_name.to_string(), used_namespace)
}
