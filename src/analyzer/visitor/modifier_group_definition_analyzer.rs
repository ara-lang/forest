use std::collections::hash_map::Entry;

use ara_parser::tree::definition::class::ClassDefinition;
use ara_parser::tree::definition::class::ClassDefinitionMember;
use ara_parser::tree::definition::constant::ClassishConstantDefinition;
use ara_parser::tree::definition::function::AbstractMethodDefinition;
use ara_parser::tree::definition::function::ConcreteMethodDefinition;
use ara_parser::tree::definition::modifier::ModifierDefinition;
use ara_parser::tree::definition::property::PropertyDefinition;
use ara_parser::tree::definition::property::PropertyEntryDefinition;
use ara_parser::tree::identifier::Identifier;
use ara_parser::tree::token::Keyword;
use ara_parser::tree::variable::Variable;
use rustc_hash::FxHashMap;

use ara_parser::tree::definition::modifier::ModifierGroupDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct ModifierGroupDefinitionAnalyzer;

impl ModifierGroupDefinitionAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for ModifierGroupDefinitionAnalyzer {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(modifiers) = downcast::<ModifierGroupDefinition>(node) {
            let mut map = FxHashMap::default();

            for modifier in &modifiers.modifiers {
                let name = modifier.to_string().to_lowercase();
                match map.entry(name) {
                    Entry::Occupied(entry) => {
                        issues.push(duplicate_modifier(
                            source,
                            entry.key(),
                            entry.get(),
                            modifier,
                        ));
                    }
                    Entry::Vacant(entry) => {
                        entry.insert((modifier.initial_position(), modifier.final_position()));
                    }
                }
            }
        }

        if let Some(class) = downcast::<ClassDefinition>(node) {
            let mut abstract_class: Option<&Keyword> = None;
            let mut final_class: Option<&Keyword> = None;
            let mut readonly_class: Option<&Keyword> = None;

            for modifier in &class.modifiers.modifiers {
                match modifier {
                    ModifierDefinition::Readonly(keyword) => {
                        readonly_class = Some(keyword);
                    }
                    ModifierDefinition::Final(keyword) => {
                        if let Some(abstract_class) = abstract_class {
                            issues.push(abstract_modifier_cannot_be_used_on_final_class(
                                source,
                                abstract_class,
                                keyword,
                                &class.name,
                            ));
                        } else {
                            final_class = Some(keyword);
                        }
                    }
                    ModifierDefinition::Abstract(keyword) => {
                        if let Some(r#final) = final_class {
                            issues.push(abstract_modifier_cannot_be_used_on_final_class(
                                source,
                                keyword,
                                r#final,
                                &class.name,
                            ));
                        } else {
                            abstract_class = Some(keyword);
                        }
                    }
                    ModifierDefinition::Public(keyword)
                    | ModifierDefinition::Protected(keyword)
                    | ModifierDefinition::Private(keyword)
                    | ModifierDefinition::Static(keyword) => {
                        issues.push(modifier_cannot_be_used_on_class(
                            source,
                            keyword,
                            &class.name,
                        ));
                    }
                }
            }

            for member in &class.body.members {
                match &member {
                    ClassDefinitionMember::Constant(constant) => {
                        let mut visibility: Option<&Keyword> = None;

                        for modifier in &constant.modifiers.modifiers {
                            match modifier {
                                ModifierDefinition::Public(keyword)
                                | ModifierDefinition::Protected(keyword)
                                | ModifierDefinition::Private(keyword) => {
                                    if let Some(previous) = visibility {
                                        issues.push(duplicate_visibility_modifier(
                                            source, keyword, previous,
                                        ));
                                    } else {
                                        visibility = Some(keyword);
                                    }
                                }
                                ModifierDefinition::Static(keyword)
                                | ModifierDefinition::Readonly(keyword)
                                | ModifierDefinition::Abstract(keyword) => {
                                    issues
                                        .push(modifier_cannot_be_used_on_constant(source, keyword));
                                }
                                ModifierDefinition::Final(keyword) => {
                                    if let Some(r#final) = final_class {
                                        issues.push(unnecessary_modifier(
                                            source,
                                            keyword,
                                            r#final,
                                            &class.name,
                                        ));
                                    }
                                }
                            }
                        }

                        if let None = visibility {
                            issues.push(missing_visibility_modifier(source, &class.name, member));
                        }
                    }
                    ClassDefinitionMember::Property(property) => {
                        let mut visibility: Option<&Keyword> = None;
                        let mut readonly: Option<&Keyword> = None;
                        let mut r#static: Option<&Keyword> = None;

                        for modifier in &property.modifiers.modifiers {
                            match modifier {
                                ModifierDefinition::Public(keyword)
                                | ModifierDefinition::Protected(keyword)
                                | ModifierDefinition::Private(keyword) => {
                                    if let Some(previous) = visibility {
                                        issues.push(duplicate_visibility_modifier(
                                            source, keyword, previous,
                                        ));
                                    } else {
                                        visibility = Some(keyword);
                                    }
                                }
                                ModifierDefinition::Static(keyword) => {
                                    if let Some(readonly) = readonly.or(readonly_class) {
                                        issues.push(
                                            static_modifier_cannot_be_used_with_readonly_modifier(
                                                source, keyword, readonly,
                                            ),
                                        );
                                    } else {
                                        r#static = Some(keyword);
                                    }
                                }
                                ModifierDefinition::Readonly(keyword) => {
                                    if let Some(readonly) = readonly_class {
                                        issues.push(unnecessary_modifier(
                                            source,
                                            keyword,
                                            readonly,
                                            &class.name,
                                        ));
                                    }

                                    if let Some(r#static) = r#static {
                                        issues.push(
                                            static_modifier_cannot_be_used_with_readonly_modifier(
                                                source, r#static, keyword,
                                            ),
                                        );
                                    } else {
                                        readonly = Some(keyword);
                                    }
                                }
                                ModifierDefinition::Abstract(keyword) => {
                                    issues.push(modifier_cannot_be_used_on_property(
                                        source,
                                        keyword,
                                        &class.name,
                                        property.entry.variable(),
                                    ));
                                }
                                ModifierDefinition::Final(keyword) => {
                                    if let Some(r#final) = final_class {
                                        issues.push(unnecessary_modifier(
                                            source,
                                            keyword,
                                            r#final,
                                            &class.name,
                                        ));
                                    }
                                }
                            }
                        }

                        if let PropertyEntryDefinition::Initialized { equals, .. } = property.entry
                        {
                            if let Some(readonly) = readonly.or(readonly_class) {
                                issues.push(
                                    readonly_modifier_cannot_be_used_with_initialized_property(
                                        source,
                                        readonly,
                                        equals,
                                        &class.name,
                                        &property.entry,
                                    ),
                                );
                            }
                        }

                        if let None = visibility {
                            issues.push(missing_visibility_modifier(source, &class.name, member));
                        }
                    }
                    ClassDefinitionMember::AbstractMethod(AbstractMethodDefinition {
                        modifiers,
                        ..
                    })
                    | ClassDefinitionMember::ConcreteMethod(ConcreteMethodDefinition {
                        modifiers,
                        ..
                    }) => {
                        let mut visibility: Option<&Keyword> = None;
                        let mut r#abstract: Option<&Keyword> = None;
                        let mut r#static: Option<&Keyword> = None;
                        let mut r#final: Option<&Keyword> = None;

                        for modifier in &modifiers.modifiers {
                            match modifier {
                                ModifierDefinition::Public(keyword)
                                | ModifierDefinition::Protected(keyword)
                                | ModifierDefinition::Private(keyword) => {
                                    if let Some(previous) = visibility {
                                        issues.push(duplicate_visibility_modifier(
                                            source, keyword, previous,
                                        ));
                                    } else {
                                        visibility = Some(keyword);
                                    }
                                }
                                ModifierDefinition::Readonly(keyword) => {
                                    issues.push(modifier_cannot_be_used_on_method(
                                        source,
                                        keyword,
                                        &class.name,
                                        member,
                                    ));
                                }
                                ModifierDefinition::Static(keyword) => {
                                    r#static = Some(keyword);
                                }
                                ModifierDefinition::Abstract(keyword) => {
                                    if let None = abstract_class {
                                        issues.push(
                                            abstract_method_modifier_on_non_abstract_class(
                                                source,
                                                keyword,
                                                &class.name,
                                            ),
                                        );
                                    } else if let Some(r#final) = r#final {
                                        issues.push(
                                            abstract_modifier_cannot_be_used_with_final_modifier(
                                                source,
                                                keyword,
                                                r#final,
                                                &class.name,
                                            ),
                                        );
                                    } else {
                                        r#abstract = Some(keyword);
                                    }
                                }
                                ModifierDefinition::Final(keyword) => {
                                    if let Some(r#final) = final_class {
                                        issues.push(unnecessary_modifier(
                                            source,
                                            keyword,
                                            r#final,
                                            &class.name,
                                        ));
                                    } else {
                                        r#final = Some(keyword);
                                    }
                                }
                            }
                        }

                        if let None = visibility {
                            issues.push(missing_visibility_modifier(source, &class.name, member));
                        }
                    }
                    ClassDefinitionMember::AbstractConstructor(_) => {}
                    ClassDefinitionMember::ConcreteConstructor(_) => {}
                }
            }
        }

        issues
    }
}

#[inline(always)]
fn duplicate_modifier(
    source: &str,
    name: &str,
    previous: &(usize, usize),
    modifier: &dyn Node,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::NoDuplicateModifier,
        format!("the modifier `{}` is defined multiple times", name),
    )
    .with_source(
        source,
        modifier.initial_position(),
        modifier.final_position(),
    )
    .with_annotation(
        Annotation::secondary(source, previous.0, previous.1).with_message(format!(
            "previous definition of the modifier `{}` here",
            name
        )),
    )
    .with_note(format!("help: remove the duplicate `{}` modifier", name))
}

#[inline(always)]
fn duplicate_visibility_modifier(source: &str, keyword: &Keyword, previous: &Keyword) -> Issue {
    Issue::error(
        AnalyzerIssueCode::NoDuplicateVisibilityModifier,
        format!(
            "duplicate property visibility modifier definition ( `{}` )",
            keyword.value,
        ),
    )
    .with_source(source, keyword.initial_position(), keyword.final_position())
    .with_annotation(
        Annotation::secondary(
            source,
            previous.initial_position(),
            previous.final_position(),
        )
        .with_message(format!(
            "previous visibility modifier definition ( `{}` ) here",
            previous.value
        )),
    )
    .with_note(format!(
        "help: remove either the `{}` visibility modifier",
        keyword.value
    ))
}

#[inline(always)]
fn static_modifier_cannot_be_used_with_readonly_modifier(
    source: &str,
    r#static: &Keyword,
    readonly: &Keyword,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::StaticModifierCannotBeUsedWithReadonlyModifier,
        "the `static` modifier cannot be used with the `readonly` modifier",
    )
    .with_source(
        source,
        r#static.initial_position(),
        r#static.final_position(),
    )
    .with_annotation(
        Annotation::secondary(
            source,
            readonly.initial_position(),
            readonly.final_position(),
        )
        .with_message("definition of the `readonly` modifier here"),
    )
    .with_note("help: remove either the `static` or the `readonly` modifier")
}

#[inline(always)]
fn missing_visibility_modifier(source: &str, classname: &Identifier, member: &dyn Node) -> Issue {
    Issue::error(
        AnalyzerIssueCode::MissingVisibilityModifier,
        "missing visibility modifier",
    )
    .with_source(source, member.initial_position(), member.final_position())
    .with_source(
        source,
        classname.initial_position(),
        classname.final_position(),
    )
    .with_note("help: add a visibility modifier, such as `public`, `protected`, or `private`")
}

#[inline(always)]
fn modifier_cannot_be_used_on_property(
    source: &str,
    modifier: &Keyword,
    classname: &Identifier,
    property: &Variable,
) -> Issue {
    let name = modifier.value.to_string();

    Issue::error(
        AnalyzerIssueCode::ModifierCannotBeUsedOnProperty,
        format!("the `{}` modifier cannot be used on a property", name),
    )
    .with_source(
        source,
        modifier.initial_position(),
        modifier.final_position(),
    )
    .with_annotation(Annotation::secondary(
        source,
        property.initial_position(),
        property.final_position(),
    ))
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note(format!("help: remove the `{}` modifier", name))
}

#[inline(always)]
fn modifier_cannot_be_used_on_constant(source: &str, modifier: &Keyword) -> Issue {
    let name = modifier.value.to_string();

    Issue::error(
        AnalyzerIssueCode::ModifierCannotBeUsedOnConstant,
        format!("the `{}` modifier cannot be used on a constant", name),
    )
    .with_source(
        source,
        modifier.initial_position(),
        modifier.final_position(),
    )
    .with_note(format!("help: remove the `{}` modifier", name))
}

#[inline(always)]
fn modifier_cannot_be_used_on_class(
    source: &str,
    modifier: &Keyword,
    classname: &Identifier,
) -> Issue {
    let name = modifier.value.to_string();

    Issue::error(
        AnalyzerIssueCode::ModifierCannotBeUsedOnClass,
        format!("the `{}` modifier cannot be used on a class", name),
    )
    .with_source(
        source,
        modifier.initial_position(),
        modifier.final_position(),
    )
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note(format!("help: remove the `{}` modifier", name))
}

#[inline(always)]
fn modifier_cannot_be_used_on_method(
    source: &str,
    modifier: &Keyword,
    classname: &Identifier,
    member: &dyn Node,
) -> Issue {
    let name = modifier.value.to_string();

    Issue::error(
        AnalyzerIssueCode::ModifierCannotBeUsedOnMethod,
        format!("the `{}` modifier cannot be used on a method", name),
    )
    .with_source(
        source,
        modifier.initial_position(),
        modifier.final_position(),
    )
    .with_annotation(Annotation::secondary(
        source,
        member.initial_position(),
        member.final_position(),
    ))
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note(format!("help: remove the `{}` modifier", name))
}

#[inline(always)]
fn abstract_modifier_cannot_be_used_on_final_class(
    source: &str,
    r#abstract: &Keyword,
    r#final: &Keyword,
    classname: &Identifier,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::AbstractModifierCannotBeUsedOnFinalClass,
        "the `abstract` modifier cannot be used on a `final` class",
    )
    .with_source(
        source,
        r#abstract.initial_position(),
        r#abstract.final_position(),
    )
    .with_annotation(
        Annotation::secondary(source, r#final.initial_position(), r#final.final_position())
            .with_message("definition of the `final` modifier here"),
    )
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note("help: remove either the `abstract` or the `final` modifier")
}

fn unnecessary_modifier(
    source: &str,
    modifier: &Keyword,
    previous: &Keyword,
    classname: &Identifier,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::UnnecessaryModifier,
        format!("the `{}` modifier is unnecessary", modifier.value),
    )
    .with_source(
        source,
        modifier.initial_position(),
        modifier.final_position(),
    )
    .with_annotation(Annotation::secondary(
        source,
        previous.initial_position(),
        previous.final_position(),
    ))
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note(format!("help: remove the `{}` modifier", modifier.value))
}

fn readonly_modifier_cannot_be_used_with_initialized_property(
    source: &str,
    r#readonly: &Keyword,
    equals: usize,
    classname: &Identifier,
    entry: &PropertyEntryDefinition,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::ReadonlyModifierCannotBeUsedWithInitializedProperty,
        "the `readonly` modifier cannot be used with an initialized property",
    )
    .with_source(
        source,
        r#readonly.initial_position(),
        r#readonly.final_position(),
    )
    .with_annotation(Annotation::secondary(
        source,
        equals,
        entry.final_position(),
    ))
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note("help: remove the `readonly` modifier, or remove the initializer")
}

fn abstract_method_modifier_on_non_abstract_class(
    source: &str,
    r#abstract: &Keyword,
    classname: &Identifier,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::AbstractMethodModifierOnNonAbstractClass,
        "the `abstract` method modifier can only be used on an `abstract` class",
    )
    .with_source(
        source,
        r#abstract.initial_position(),
        r#abstract.final_position(),
    )
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note("help: remove the `abstract` modifier, or make the class `abstract`")
}

fn abstract_modifier_cannot_be_used_with_final_modifier(
    source: &str,
    r#abstract: &Keyword,
    r#final: &Keyword,
    classname: &Identifier,
) -> Issue {
    Issue::error(
        AnalyzerIssueCode::AbstractModifierCannotBeUsedWithFinalModifier,
        "the `abstract` modifier cannot be used with the `final` modifier",
    )
    .with_source(
        source,
        r#abstract.initial_position(),
        r#abstract.final_position(),
    )
    .with_annotation(
        Annotation::secondary(source, r#final.initial_position(), r#final.final_position())
            .with_message("definition of the `final` modifier here"),
    )
    .with_annotation(Annotation::secondary(
        source,
        classname.initial_position(),
        classname.final_position(),
    ))
    .with_note("help: remove the `abstract` modifier")
}
