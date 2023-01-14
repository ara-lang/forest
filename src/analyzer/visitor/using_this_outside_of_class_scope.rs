use ara_parser::tree::definition::class::ClassDefinition;
use ara_parser::tree::definition::function::ConstructorParameterDefinition;
use ara_parser::tree::definition::function::FunctionLikeParameterDefinition;
use ara_parser::tree::definition::interface::InterfaceDefinition;
use ara_parser::tree::definition::property::PropertyDefinition;
use ara_parser::tree::definition::r#enum::EnumDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::identifier::Identifier;
use ara_parser::tree::variable::Variable;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct UsingThisOutsideOfClassContext;

impl UsingThisOutsideOfClassContext {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for UsingThisOutsideOfClassContext {
    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        if let Some(identifier) = downcast::<Identifier>(node) {
            let name = identifier.value.to_string();
            let lowercase_name = name.to_lowercase();

            if lowercase_name == "self" {
                let scope = get_scope(ancestry);

                if scope == Scope::Global {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseSelfOutsideOfClassScope,
                        "cannot use `self` outside of a class scope.",
                        source,
                        identifier.initial_position(),
                        identifier.final_position(),
                    )];
                }
            }

            if lowercase_name == "static" {
                let scope = get_scope(ancestry);

                if scope == Scope::Global {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseStaticOutsideOfClassScope,
                        "cannot use `static` outside of a class scope.",
                        source,
                        identifier.initial_position(),
                        identifier.final_position(),
                    )];
                }
            }

            if lowercase_name == "parent" {
                let scope = get_scope(ancestry);

                if scope == Scope::Global {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseParentOutsideOfClassScope,
                        "cannot use `parent` outside of a class scope.",
                        source,
                        identifier.initial_position(),
                        identifier.final_position(),
                    )];
                }

                if let Scope::Classish(from, to) = scope {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseParentWhenCurrentTypeScopeHasNoParent,
                        "cannot use `parent` when current type scope has no parent.",
                        source,
                        identifier.initial_position(),
                        identifier.final_position(),
                    )
                    .with_annotation(Annotation::secondary(source, from, to))];
                }
            }

            return vec![];
        }

        if let Some(variable) = downcast::<Variable>(node) {
            let name = variable.name.to_string();
            let lowercase_name = name.to_lowercase();

            if lowercase_name == "$this" {
                let ancestry_len = ancestry.len();

                if let Some(_) =
                    downcast::<FunctionLikeParameterDefinition>(ancestry[ancestry_len - 1])
                {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseThisAsParameter,
                        "cannot use `$this` as a parameter",
                        source,
                        variable.initial_position(),
                        variable.final_position(),
                    )];
                }

                if let Some(_) =
                    downcast::<ConstructorParameterDefinition>(ancestry[ancestry_len - 1])
                {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseThisAsParameter,
                        "cannot use `$this` as a constructor parameter",
                        source,
                        variable.initial_position(),
                        variable.final_position(),
                    )];
                }

                if let Some(_) = downcast::<PropertyDefinition>(ancestry[ancestry_len - 2]) {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseThisAsProperty,
                        "cannot use `$this` as a property",
                        source,
                        variable.initial_position(),
                        variable.final_position(),
                    )];
                }

                let scope = get_scope(ancestry);

                if scope == Scope::Global {
                    return vec![Issue::error(
                        AnalyzerIssueCode::CannotUseThisOutsideOfClassScope,
                        "cannot use `$this` outside of a class scope.",
                        source,
                        variable.initial_position(),
                        variable.final_position(),
                    )];
                }
            }
        }

        vec![]
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Scope {
    Global,
    Classish(usize, usize),
    ClassishWithParent,
}

fn get_scope(ancestry: &Vec<&dyn Node>) -> Scope {
    // we start looking from the outter-most, because that's where the class definition would be.
    for node in ancestry {
        if let Some(definition) = downcast::<ClassDefinition>(*node) {
            if definition.extends.is_some() {
                return Scope::ClassishWithParent;
            }

            return Scope::Classish(definition.initial_position(), definition.final_position());
        }

        if let Some(definition) = downcast::<InterfaceDefinition>(*node) {
            if definition.extends.is_some() {
                return Scope::ClassishWithParent;
            }

            return Scope::Classish(definition.initial_position(), definition.final_position());
        }

        if let Some(definition) = downcast::<EnumDefinition>(*node) {
            return Scope::Classish(definition.initial_position(), definition.final_position());
        }
    }

    return Scope::Global;
}
