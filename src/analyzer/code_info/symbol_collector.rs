use rustc_hash::FxHashMap;

use ara_parser::tree::definition::r#enum::EnumDefinition;
use ara_parser::tree::definition::Definition;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::code_info::symbol_storage::CodeBaseSymbols;
use crate::analyzer::code_info::symbol_storage::Symbol;
use crate::analyzer::issue::AnalyzerIssueCode;

pub fn collect_symbols(map: &FxHashMap<String, Vec<Definition>>) -> (CodeBaseSymbols, Vec<Issue>) {
    let mut storage = CodeBaseSymbols::new();
    let mut issues = Vec::new();

    for file_map in map {
        let (file, definitions) = file_map;

        let mut namespace = None;
        let mut uses = Vec::new();
        for definition in definitions {
            match &definition {
                Definition::Namespace(definition) => {
                    namespace = Some(definition.name.value.to_string());
                }
                Definition::Function(function) => {
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, function.name.value),
                        None => function.name.value.to_string(),
                    };

                    if let Some(symbol) = storage.get_function(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            function.name.value.to_string(),
                            file,
                            function.initial_position(),
                            function.return_type.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_function(
                            fq_name,
                            file.to_string(),
                            (function.initial_position(), function.final_position()),
                        );
                    }
                }
                Definition::Interface(interface) => {
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, interface.name.value),
                        None => interface.name.value.to_string(),
                    };

                    if let Some(symbol) = storage.get_classish(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            interface.name.value.to_string(),
                            file,
                            interface.initial_position(),
                            interface.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_interface(
                            fq_name,
                            file.to_string(),
                            (interface.initial_position(), interface.final_position()),
                        );
                    }
                }
                Definition::Class(class) => {
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, class.name.value),
                        None => class.name.value.to_string(),
                    };

                    if let Some(symbol) = storage.get_classish(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            class.name.value.to_string(),
                            file,
                            class.initial_position(),
                            class.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_class(
                            fq_name,
                            file.to_string(),
                            (class.initial_position(), class.final_position()),
                        );
                    }
                }
                Definition::Enum(r#enum) => match r#enum.as_ref() {
                    EnumDefinition::Backed(backed_enum) => {
                        let fq_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace, backed_enum.name.value),
                            None => backed_enum.name.value.to_string(),
                        };

                        if let Some(symbol) = storage.get_classish(&fq_name) {
                            let issue = duplicate_item_issue(
                                &symbol,
                                backed_enum.name.value.to_string(),
                                file,
                                backed_enum.initial_position(),
                                backed_enum.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            storage.add_enum(
                                fq_name,
                                file.to_string(),
                                (backed_enum.initial_position(), backed_enum.final_position()),
                            );
                        }
                    }
                    EnumDefinition::Unit(unit_enum) => {
                        let fq_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace, unit_enum.name.value),
                            None => unit_enum.name.value.to_string(),
                        };

                        if let Some(symbol) = storage.get_classish(&fq_name) {
                            let issue = duplicate_item_issue(
                                &symbol,
                                unit_enum.name.value.to_string(),
                                file,
                                unit_enum.initial_position(),
                                unit_enum.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            storage.add_enum(
                                fq_name,
                                file.to_string(),
                                (unit_enum.initial_position(), unit_enum.final_position()),
                            );
                        }
                    }
                },
                Definition::TypeAlias(type_alias) => {
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, type_alias.name.name.value),
                        None => type_alias.name.name.value.to_string(),
                    };

                    if let Some(symbol) = storage.get_classish(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            type_alias.name.name.value.to_string(),
                            file,
                            type_alias.initial_position(),
                            type_alias.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_type_alias(
                            fq_name,
                            file.to_string(),
                            (type_alias.initial_position(), type_alias.final_position()),
                        );
                    }
                }
                Definition::Constant(constant) => {
                    for entry in &constant.entries.inner {
                        let fq_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace, entry.name.value),
                            None => entry.name.value.to_string(),
                        };

                        if let Some(symbol) = storage.get_constant(&fq_name) {
                            let issue = duplicate_item_issue(
                                &symbol,
                                entry.name.value.to_string(),
                                file,
                                entry.initial_position(),
                                entry.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            storage.add_constant(
                                fq_name,
                                file.to_string(),
                                (entry.initial_position(), entry.final_position()),
                            );
                        }
                    }
                }
                Definition::Use(r#use) => {
                    uses.push(r#use);
                }
            }
        }
    }

    (storage, issues)
}

fn duplicate_item_issue(
    previous: &Symbol,
    name: String,
    source: &str,
    from: usize,
    to: usize,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::DuplicateItemDefinition,
        format!("the item `{}` is defined multiple times", previous.name),
        source,
        from,
        to,
    )
    .with_annotation(
        Annotation::secondary(&previous.source, previous.position.0, previous.position.1)
            .with_message(format!("previous definition of the item `{}` here", name)),
    )
}
