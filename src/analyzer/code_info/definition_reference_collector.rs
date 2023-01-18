use ara_parser::tree::identifier::Identifier;
use rustc_hash::FxHashMap;

use ara_parser::tree::definition::r#enum::BackedEnumTypeDefinition;
use ara_parser::tree::definition::r#enum::EnumDefinition;
use ara_parser::tree::definition::r#use::UseDefinition;
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

    for source in map {
        let (source_name, definitions) = source;

        let mut namespace = None;
        for definition in definitions {
            match &definition {
                Definition::Namespace(definition) => {
                    namespace = Some((
                        definition.name.value.to_string(),
                        (definition.initial_position(), definition.final_position()),
                    ));
                }
                Definition::Function(function) => {
                    let name = function.name.value.to_string();
                    let qualified_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace.0, name),
                        None => name.clone(),
                    };

                    if let Some(symbol) = storage.get_function(&qualified_name) {
                        let issue = duplicate_item_issue(
                            symbol,
                            source_name,
                            function.initial_position(),
                            function.return_type.final_position(),
                        );

                        issues.push(issue);
                    } else if let Some(definition) =
                        storage.get_function_name_in_source(source_name, &name)
                    {
                        let issue = name_already_in_use(
                            definition,
                            source_name,
                            function.initial_position(),
                            function.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_function(
                            name,
                            qualified_name,
                            source_name.to_string(),
                            (function.initial_position(), function.final_position()),
                        );
                    }
                }
                Definition::Interface(interface) => {
                    let name = interface.name.value.to_string();
                    let qualified_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace.0, name),
                        None => name.clone(),
                    };

                    if is_name_reserved(&name) {
                        let issue = name_is_reserved(
                            &interface.name,
                            source_name,
                            interface.initial_position(),
                            interface.final_position(),
                        );

                        issues.push(issue);
                    } else if let Some(definition) = storage.get_type(&qualified_name) {
                        let issue = duplicate_item_issue(
                            definition,
                            source_name,
                            interface.initial_position(),
                            interface.final_position(),
                        );

                        issues.push(issue);
                    } else if let Some(definition) =
                        storage.get_type_name_in_source(source_name, &name)
                    {
                        let issue = name_already_in_use(
                            definition,
                            source_name,
                            interface.initial_position(),
                            interface.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_interface(
                            name,
                            qualified_name,
                            source_name.to_string(),
                            (interface.initial_position(), interface.final_position()),
                        );
                    }
                }
                Definition::Class(class) => {
                    let name = class.name.value.to_string();
                    let qualified_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace.0, name),
                        None => name.clone(),
                    };

                    if is_name_reserved(&name) {
                        let issue = name_is_reserved(
                            &class.name,
                            source_name,
                            class.initial_position(),
                            class.final_position(),
                        );

                        issues.push(issue);
                    } else if let Some(symbol) = storage.get_type(&qualified_name) {
                        let issue = duplicate_item_issue(
                            symbol,
                            source_name,
                            class.initial_position(),
                            class.final_position(),
                        );

                        issues.push(issue);
                    } else if let Some(definition) =
                        storage.get_type_name_in_source(source_name, &name)
                    {
                        let issue = name_already_in_use(
                            definition,
                            source_name,
                            class.initial_position(),
                            class.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_class(
                            name,
                            qualified_name,
                            source_name.to_string(),
                            (class.initial_position(), class.final_position()),
                        );
                    }
                }
                Definition::Enum(r#enum) => match r#enum.as_ref() {
                    EnumDefinition::Backed(backed_enum) => {
                        let name = backed_enum.name.value.to_string();
                        let qualified_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace.0, name),
                            None => name.clone(),
                        };

                        if is_name_reserved(&name) {
                            let issue = name_is_reserved(
                                &backed_enum.name,
                                source_name,
                                backed_enum.initial_position(),
                                backed_enum.final_position(),
                            );

                            issues.push(issue);
                        } else if let Some(definition) = storage.get_type(&qualified_name) {
                            let issue = duplicate_item_issue(
                                definition,
                                source_name,
                                backed_enum.initial_position(),
                                backed_enum.final_position(),
                            );

                            issues.push(issue);
                        } else if let Some(definition) =
                            storage.get_type_name_in_source(source_name, &name)
                        {
                            let issue = name_already_in_use(
                                definition,
                                source_name,
                                backed_enum.initial_position(),
                                backed_enum.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            match backed_enum.backed_type {
                                BackedEnumTypeDefinition::String(_, _) => {
                                    storage.add_string_backed_enum(
                                        name,
                                        qualified_name,
                                        source_name.to_string(),
                                        (
                                            backed_enum.initial_position(),
                                            backed_enum.final_position(),
                                        ),
                                    );
                                }
                                BackedEnumTypeDefinition::Int(_, _) => {
                                    storage.add_int_backed_enum(
                                        name,
                                        qualified_name,
                                        source_name.to_string(),
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
                        let name = unit_enum.name.value.to_string();
                        let qualified_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace.0, name),
                            None => name.clone(),
                        };

                        if is_name_reserved(&name) {
                            let issue = name_is_reserved(
                                &unit_enum.name,
                                source_name,
                                unit_enum.initial_position(),
                                unit_enum.final_position(),
                            );

                            issues.push(issue);
                        } else if let Some(symbol) = storage.get_type(&qualified_name) {
                            let issue = duplicate_item_issue(
                                symbol,
                                source_name,
                                unit_enum.initial_position(),
                                unit_enum.final_position(),
                            );

                            issues.push(issue);
                        } else if let Some(definition) =
                            storage.get_type_name_in_source(source_name, &name)
                        {
                            let issue = name_already_in_use(
                                definition,
                                source_name,
                                unit_enum.initial_position(),
                                unit_enum.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            storage.add_unit_enum(
                                name,
                                qualified_name,
                                source_name.to_string(),
                                (unit_enum.initial_position(), unit_enum.final_position()),
                            );
                        }
                    }
                },
                Definition::TypeAlias(type_alias) => {
                    let name = type_alias.name.name.value.to_string();
                    let qualified_name = match &namespace {
                        Some(namespace) => format!("{}\\{}", namespace.0, name),
                        None => name.clone(),
                    };

                    if is_name_reserved(&name) {
                        let issue = name_is_reserved(
                            &type_alias.name.name,
                            source_name,
                            type_alias.initial_position(),
                            type_alias.final_position(),
                        );

                        issues.push(issue);
                    } else if let Some(definition) = storage.get_type(&qualified_name) {
                        let issue = duplicate_item_issue(
                            definition,
                            source_name,
                            type_alias.initial_position(),
                            type_alias.final_position(),
                        );

                        issues.push(issue);
                    } else if let Some(definition) =
                        storage.get_type_name_in_source(source_name, &name)
                    {
                        let issue = name_already_in_use(
                            definition,
                            source_name,
                            type_alias.initial_position(),
                            type_alias.final_position(),
                        );

                        issues.push(issue);
                    } else {
                        storage.add_type_alias(
                            name,
                            qualified_name,
                            source_name.to_string(),
                            (type_alias.initial_position(), type_alias.final_position()),
                        );
                    }
                }
                Definition::Constant(constant) => {
                    for entry in &constant.entries.inner {
                        let name = entry.name.value.to_string();
                        let qualified_name = match &namespace {
                            Some(namespace) => format!("{}\\{}", namespace.0, name),
                            None => name.clone(),
                        };

                        if let Some(symbol) = storage.get_constant(&qualified_name) {
                            let issue = duplicate_item_issue(
                                symbol,
                                source_name,
                                entry.initial_position(),
                                entry.final_position(),
                            );

                            issues.push(issue);
                        } else if let Some(definition) =
                            storage.get_constant_name_in_source(source_name, &name)
                        {
                            let issue = name_already_in_use(
                                definition,
                                source_name,
                                entry.initial_position(),
                                entry.final_position(),
                            );

                            issues.push(issue);
                        } else {
                            storage.add_constant(
                                name,
                                qualified_name,
                                source_name.to_string(),
                                (entry.initial_position(), entry.final_position()),
                            );
                        }
                    }
                }
                Definition::Use(use_definition) => {
                    let mut redundant_import = false;
                    let mut redundant_alias = None;
                    let mut duplicate_import = None;
                    let mut duplicate_import_under_alias = None;

                    match use_definition.as_ref() {
                        UseDefinition::Default { name, alias, .. } => {
                            let full_name = name.value.to_string();
                            let (mut used_name, used_namespace) =
                                get_name_and_namespace(&full_name);

                            if is_name_reserved(&used_name) {
                                let issue = name_is_reserved(
                                    name,
                                    source_name,
                                    use_definition.initial_position(),
                                    use_definition.final_position(),
                                );

                                issues.push(issue);
                            } else {
                                if let Some(alias) = alias {
                                    let aliased_name = &alias.alias;
                                    if aliased_name.value.to_string() == used_name {
                                        redundant_alias = Some((name.final_position(), alias));
                                    }

                                    used_name = aliased_name.value.to_string();

                                    if is_name_reserved(&used_name) {
                                        let issue = name_is_reserved(
                                            aliased_name,
                                            source_name,
                                            use_definition.initial_position(),
                                            use_definition.final_position(),
                                        );

                                        issues.push(issue);
                                    }
                                } else if let Some(last_namespace) = &namespace {
                                    if used_namespace == last_namespace.0 {
                                        redundant_import = true;
                                    }
                                }

                                match storage.get_type_name_in_source(source_name, &used_name) {
                                    Some(def) => {
                                        duplicate_import = Some(def.clone());
                                    }
                                    None => {
                                        if let Some(def) =
                                            storage.get_used_type_in_source(source_name, &full_name)
                                        {
                                            duplicate_import_under_alias = Some(def.clone());
                                        }

                                        storage.add_use(
                                            used_name,
                                            full_name,
                                            source_name.to_string(),
                                            (
                                                use_definition.initial_position(),
                                                use_definition.final_position(),
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                        UseDefinition::Function { name, alias, .. } => {
                            let full_name = name.value.to_string();
                            let (mut used_name, used_namespace) =
                                get_name_and_namespace(&full_name);

                            if is_name_reserved(&used_name) {
                                let issue = name_is_reserved(
                                    name,
                                    source_name,
                                    use_definition.initial_position(),
                                    use_definition.final_position(),
                                );

                                issues.push(issue);
                            } else {
                                if let Some(alias) = alias {
                                    let aliased_name = &alias.alias;
                                    if aliased_name.value.to_string() == used_name {
                                        redundant_alias = Some((name.final_position(), alias));
                                    }

                                    used_name = aliased_name.value.to_string();

                                    if is_name_reserved(&used_name) {
                                        let issue = name_is_reserved(
                                            aliased_name,
                                            source_name,
                                            use_definition.initial_position(),
                                            use_definition.final_position(),
                                        );

                                        issues.push(issue);
                                    }
                                } else if let Some(last_namespace) = &namespace {
                                    if used_namespace == last_namespace.0 {
                                        redundant_import = true;
                                    }
                                }

                                match storage.get_function_name_in_source(source_name, &used_name) {
                                    Some(def) => {
                                        duplicate_import = Some(def.clone());
                                    }
                                    None => {
                                        if let Some(def) = storage
                                            .get_used_function_in_source(source_name, &full_name)
                                        {
                                            duplicate_import_under_alias = Some(def.clone());
                                        }

                                        storage.add_use_function(
                                            used_name,
                                            full_name,
                                            source_name.to_string(),
                                            (
                                                use_definition.initial_position(),
                                                use_definition.final_position(),
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                        UseDefinition::Constant { name, alias, .. } => {
                            let full_name = name.value.to_string();
                            let (mut used_name, used_namespace) =
                                get_name_and_namespace(&full_name);

                            if is_name_reserved(&used_name) {
                                let issue = name_is_reserved(
                                    name,
                                    source_name,
                                    use_definition.initial_position(),
                                    use_definition.final_position(),
                                );

                                issues.push(issue);
                            } else {
                                if let Some(alias) = alias {
                                    let aliased_name = &alias.alias;
                                    if aliased_name.value.to_string() == used_name {
                                        redundant_alias = Some((name.final_position(), alias));
                                    }

                                    used_name = aliased_name.value.to_string();

                                    if is_name_reserved(&used_name) {
                                        let issue = name_is_reserved(
                                            aliased_name,
                                            source_name,
                                            use_definition.initial_position(),
                                            use_definition.final_position(),
                                        );

                                        issues.push(issue);
                                    }
                                } else if let Some(last_namespace) = &namespace {
                                    if used_namespace == last_namespace.0 {
                                        redundant_import = true;
                                    }
                                }

                                match storage.get_constant_name_in_source(source_name, &used_name) {
                                    Some(def) => {
                                        duplicate_import = Some(def.clone());
                                    }
                                    None => {
                                        if let Some(def) = storage
                                            .get_used_constant_in_source(source_name, &full_name)
                                        {
                                            duplicate_import_under_alias = Some(def.clone());
                                        }

                                        storage.add_use_constant(
                                            used_name,
                                            full_name,
                                            source_name.to_string(),
                                            (
                                                use_definition.initial_position(),
                                                use_definition.final_position(),
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    }

                    if redundant_import {
                        let mut issue = Issue::note(
                            AnalyzerIssueCode::RedundantUse,
                            "redudant use definition".to_string(),
                        )
                        .with_source(
                            source_name,
                            use_definition.initial_position(),
                            use_definition.final_position(),
                        );

                        if let Some(namespace) = &namespace {
                            issue = issue.with_annotation(
                                Annotation::secondary(source_name, namespace.1 .0, namespace.1 .1)
                                    .with_message(format!(
                                        "namespace `{}` defined here",
                                        namespace.0
                                    )),
                            );
                        }

                        issues.push(issue);
                    }

                    if let Some(definition) = duplicate_import {
                        let issue = name_already_in_use(
                            &definition,
                            source_name,
                            use_definition.initial_position(),
                            use_definition.final_position(),
                        );

                        issues.push(issue);
                    }

                    if let Some(definition) = duplicate_import_under_alias {
                        let issue = Issue::warning(
                            AnalyzerIssueCode::DuplicateUseDefinitionUnderAlias,
                            "duplicate use definition under alias".to_string(),
                        )
                        .with_source(
                            source_name,
                            use_definition.initial_position(),
                            use_definition.final_position(),
                        )
                        .with_annotation(Annotation::secondary(
                            source_name,
                            definition.position.0,
                            definition.position.1,
                        ));

                        issues.push(issue);
                    }

                    if let Some((last, redundant_alias)) = redundant_alias {
                        let issue = Issue::note(
                            AnalyzerIssueCode::RedundantUseDefinitionAlias,
                            "redundant use definition alias".to_string(),
                        )
                        .with_source(
                            source_name,
                            redundant_alias.initial_position(),
                            redundant_alias.final_position(),
                        )
                        .with_annotation(Annotation::secondary(
                            source_name,
                            use_definition.initial_position(),
                            last,
                        ));

                        issues.push(issue);
                    }
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
        format!("the item `{}` is defined multiple times", previous.name),
    )
    .with_source(source, from, to)
    .with_annotation(
        Annotation::secondary(&previous.source, previous.position.0, previous.position.1)
            .with_message(format!(
                "previous definition of the item `{}` here",
                previous.name
            )),
    )
}

fn name_already_in_use(
    previous: &DefinitionReference,
    source: &str,
    from: usize,
    to: usize,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::NameAlreadyInUse,
        format!(
            "cannot use `{}` because the name is already in use",
            previous.name
        ),
    )
    .with_source(source, from, to)
    .with_annotation(
        Annotation::secondary(source, previous.position.0, previous.position.1).with_message(
            format!("the name `{}` is already in use here", previous.name),
        ),
    )
}

fn name_is_reserved(name: &Identifier, source: &str, from: usize, to: usize) -> Issue {
    Issue::error(
        AnalyzerIssueCode::NameIsReservedTypeName,
        format!(
            "cannot use `{}` because the name is a reserved type name",
            name
        ),
    )
    .with_source(source, name.initial_position(), name.final_position())
    .with_annotation(Annotation::secondary(source, from, to))
}

fn get_name_and_namespace(fqn: &str) -> (String, String) {
    let parts = fqn.split('\\').collect::<Vec<&str>>();
    let (used_namespace, used_short_name) = parts.split_at(parts.len() - 1);
    let used_namespace = used_namespace.join("\\");
    let used_short_name = used_short_name[0];

    (used_short_name.to_string(), used_namespace)
}

fn is_name_reserved(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "iterable"
            | "void"
            | "never"
            | "float"
            | "bool"
            | "int"
            | "string"
            | "object"
            | "mixed"
            | "nonnull"
            | "resource"
    )
}
