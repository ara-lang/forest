use ara_parser::tree::definition::r#enum::BackedEnumTypeDefinition;
use rustc_hash::FxHashMap;

use ara_parser::tree::definition::r#enum::EnumDefinition;
use ara_parser::tree::definition::Definition;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::code_info::definition_reference_storage::DefinitionReference;
use crate::analyzer::code_info::definition_reference_storage::DefinitionReferenceStorage;
use crate::analyzer::issue::AnalyzerIssueCode;

pub fn collect_definitions(
    map: &FxHashMap<String, Vec<Definition>>,
) -> (DefinitionReferenceStorage, Vec<Issue>) {
    let mut storage = DefinitionReferenceStorage::new();
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
                    let unqualified_name = function.name.value.to_string();
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, unqualified_name),
                        None => unqualified_name.clone(),
                    };

                    if let Some(symbol) = storage.get_function(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            file,
                            function.initial_position(),
                            function.return_type.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_function(
                            fq_name,
                            unqualified_name,
                            file.to_string(),
                            (function.initial_position(), function.final_position()),
                        );
                    }
                }
                Definition::Interface(interface) => {
                    let unqualified_name = interface.name.value.to_string();
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, unqualified_name),
                        None => unqualified_name.clone(),
                    };

                    if let Some(symbol) = storage.get_classish(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            file,
                            interface.initial_position(),
                            interface.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_interface(
                            fq_name,
                            unqualified_name,
                            file.to_string(),
                            (interface.initial_position(), interface.final_position()),
                        );
                    }
                }
                Definition::Class(class) => {
                    let unqualified_name = class.name.value.to_string();
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, unqualified_name),
                        None => unqualified_name.clone(),
                    };

                    if let Some(symbol) = storage.get_classish(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            file,
                            class.initial_position(),
                            class.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_class(
                            fq_name,
                            unqualified_name,
                            file.to_string(),
                            (class.initial_position(), class.final_position()),
                        );
                    }
                }
                Definition::Enum(r#enum) => match r#enum.as_ref() {
                    EnumDefinition::Backed(backed_enum) => {
                        let unqualified_name = backed_enum.name.value.to_string();
                        let fq_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace, unqualified_name),
                            None => unqualified_name.clone(),
                        };

                        if let Some(symbol) = storage.get_classish(&fq_name) {
                            let issue = duplicate_item_issue(
                                &symbol,
                                file,
                                backed_enum.initial_position(),
                                backed_enum.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            match backed_enum.backed_type {
                                BackedEnumTypeDefinition::String(_, _) => {
                                    storage.add_string_backed_enum(
                                        fq_name,
                                        unqualified_name,
                                        file.to_string(),
                                        (
                                            backed_enum.initial_position(),
                                            backed_enum.final_position(),
                                        ),
                                    );
                                }
                                BackedEnumTypeDefinition::Int(_, _) => {
                                    storage.add_int_backed_enum(
                                        fq_name,
                                        unqualified_name,
                                        file.to_string(),
                                        (
                                            backed_enum.initial_position(),
                                            backed_enum.final_position(),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                    EnumDefinition::Unit(unit_enum) => {
                        let unqualified_name = unit_enum.name.value.to_string();
                        let fq_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace, unqualified_name),
                            None => unqualified_name.clone(),
                        };

                        if let Some(symbol) = storage.get_classish(&fq_name) {
                            let issue = duplicate_item_issue(
                                &symbol,
                                file,
                                unit_enum.initial_position(),
                                unit_enum.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            storage.add_unit_enum(
                                fq_name,
                                unqualified_name,
                                file.to_string(),
                                (unit_enum.initial_position(), unit_enum.final_position()),
                            );
                        }
                    }
                },
                Definition::TypeAlias(type_alias) => {
                    let unqualified_name = type_alias.name.name.value.to_string();
                    let fq_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace, unqualified_name),
                        None => unqualified_name.clone(),
                    };

                    if let Some(symbol) = storage.get_classish(&fq_name) {
                        let issue = duplicate_item_issue(
                            &symbol,
                            file,
                            type_alias.initial_position(),
                            type_alias.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_type_alias(
                            fq_name,
                            unqualified_name,
                            file.to_string(),
                            (type_alias.initial_position(), type_alias.final_position()),
                        );
                    }
                }
                Definition::Constant(constant) => {
                    for entry in &constant.entries.inner {
                        let unqualified_name = entry.name.value.to_string();
                        let fq_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace, unqualified_name),
                            None => unqualified_name.clone(),
                        };

                        if let Some(symbol) = storage.get_constant(&fq_name) {
                            let issue = duplicate_item_issue(
                                &symbol,
                                file,
                                entry.initial_position(),
                                entry.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            storage.add_constant(
                                fq_name,
                                unqualified_name,
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
    previous: &DefinitionReference,
    source: &str,
    from: usize,
    to: usize,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::DuplicateItemDefinition,
        format!(
            "the item `{}` is defined multiple times",
            previous.unqualified_name
        ),
        source,
        from,
        to,
    )
    .with_annotation(
        Annotation::secondary(&previous.source, previous.position.0, previous.position.1)
            .with_message(format!(
                "previous definition of the item `{}` here",
                previous.unqualified_name
            )),
    )
}
