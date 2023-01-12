use inflections::case;

use ara_parser::tree::definition::class::ClassDefinition;
use ara_parser::tree::definition::constant::ClassishConstantDefinition;
use ara_parser::tree::definition::constant::ConstantDefinition;
use ara_parser::tree::definition::function::AbstractMethodDefinition;
use ara_parser::tree::definition::function::ConcreteMethodDefinition;
use ara_parser::tree::definition::function::FunctionDefinition;
use ara_parser::tree::definition::function::FunctionLikeParameterDefinition;
use ara_parser::tree::definition::interface::InterfaceDefinition;
use ara_parser::tree::definition::property::PropertyDefinition;
use ara_parser::tree::definition::r#enum::EnumDefinition;
use ara_parser::tree::definition::r#type::TypeAliasDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct NamingConvention;

impl NamingConvention {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for NamingConvention {
    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        if let Some(function) = downcast::<FunctionDefinition>(node) {
            // SAFETY: We know that the function name is valid UTF-8 because the parser
            let name =
                unsafe { std::str::from_utf8_unchecked(function.name.value.bytes.as_slice()) };

            if !case::is_snake_case(name) {
                let suggestion = case::to_snake_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("function name should use `snake_case`"),
                    source.clone(),
                    function.name.initial_position(),
                    function.name.final_position(),
                )
                .with_note(format!("try renaming the function to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(class) = downcast::<ClassDefinition>(node) {
            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name = unsafe { std::str::from_utf8_unchecked(class.name.value.bytes.as_slice()) };

            if !case::is_pascal_case(name) {
                let suggestion = case::to_pascal_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("class name should use `PascalCase`"),
                    source.clone(),
                    class.name.initial_position(),
                    class.name.final_position(),
                )
                .with_note(format!("try renaming the class to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(interface) = downcast::<InterfaceDefinition>(node) {
            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name =
                unsafe { std::str::from_utf8_unchecked(interface.name.value.bytes.as_slice()) };

            if !case::is_pascal_case(name) {
                let suggestion = case::to_pascal_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("interface name should use `PascalCase`"),
                    source.clone(),
                    interface.name.initial_position(),
                    interface.name.final_position(),
                )
                .with_note(format!("try renaming the interface to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(r#enum) = downcast::<EnumDefinition>(node) {
            let identifier = match &r#enum {
                EnumDefinition::Backed(backed) => &backed.name,
                EnumDefinition::Unit(unit) => &unit.name,
            };

            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name = unsafe { std::str::from_utf8_unchecked(identifier.value.bytes.as_slice()) };

            if !case::is_pascal_case(name) {
                let suggestion = case::to_pascal_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("enum name should use `PascalCase`"),
                    source.clone(),
                    identifier.initial_position(),
                    identifier.final_position(),
                )
                .with_note(format!("try renaming the enum to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(type_alias) = downcast::<TypeAliasDefinition>(node) {
            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name = unsafe {
                std::str::from_utf8_unchecked(type_alias.name.name.value.bytes.as_slice())
            };

            if !case::is_pascal_case(name) {
                let suggestion = case::to_pascal_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("type alias name should use `PascalCase`"),
                    source.clone(),
                    type_alias.name.initial_position(),
                    type_alias.name.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source.clone(),
                    type_alias.name.initial_position(),
                    type_alias.name.final_position(),
                ))
                .with_note(format!("try renaming the type alias to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(parameter) = downcast::<FunctionLikeParameterDefinition>(node) {
            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name =
                unsafe { std::str::from_utf8_unchecked(parameter.variable.name.bytes.as_slice()) };

            let parent = ancestry.last().unwrap();

            if !case::is_snake_case(name) {
                let suggestion = case::to_snake_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("parameter name should use `snake_case`"),
                    source.clone(),
                    parameter.variable.initial_position(),
                    parameter.variable.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source,
                    parent.initial_position(),
                    parent.final_position(),
                ))
                .with_note(format!("try renaming the parameter to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(property) = downcast::<PropertyDefinition>(node) {
            let variable = property.entry.variable();

            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name = unsafe { std::str::from_utf8_unchecked(variable.name.bytes.as_slice()) };

            if !case::is_camel_case(name) {
                let suggestion = case::to_camel_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("property name should use `camelCase`"),
                    source.clone(),
                    variable.initial_position(),
                    variable.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source,
                    property.initial_position(),
                    property.final_position(),
                ))
                .with_note(format!("try renaming the property to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(constant) = downcast::<ConstantDefinition>(node) {
            for entry in &constant.entries.inner {
                // SAFETY: We know that the class name is valid UTF-8 because the parser
                let name =
                    unsafe { std::str::from_utf8_unchecked(entry.name.value.bytes.as_slice()) };

                if !case::is_constant_case(name) {
                    let suggestion = case::to_constant_case(name);

                    let issue = Issue::note(
                        AnalyzerIssueCode::NamingConventionViolation,
                        format!("constant name should use `CONSTANT_CASE`"),
                        source.clone(),
                        entry.name.initial_position(),
                        entry.name.final_position(),
                    )
                    .with_annotation(Annotation::secondary(
                        source,
                        constant.initial_position(),
                        constant.final_position(),
                    ))
                    .with_note(format!("try renaming the constant to `{}`", suggestion));

                    return vec![issue];
                }
            }
        } else if let Some(constant) = downcast::<ClassishConstantDefinition>(node) {
            for entry in &constant.entries.inner {
                // SAFETY: We know that the class name is valid UTF-8 because the parser
                let name =
                    unsafe { std::str::from_utf8_unchecked(entry.name.value.bytes.as_slice()) };

                if !case::is_constant_case(name) {
                    let suggestion = case::to_constant_case(name);

                    let issue = Issue::note(
                        AnalyzerIssueCode::NamingConventionViolation,
                        format!("class-ish constant name should use `CONSTANT_CASE`"),
                        source.clone(),
                        entry.name.initial_position(),
                        entry.name.final_position(),
                    )
                    .with_annotation(Annotation::secondary(
                        source,
                        constant.initial_position(),
                        constant.final_position(),
                    ))
                    .with_note(format!("try renaming the constant to `{}`", suggestion));

                    return vec![issue];
                }
            }
        } else if let Some(method) = downcast::<AbstractMethodDefinition>(node) {
            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name = unsafe { std::str::from_utf8_unchecked(method.name.value.bytes.as_slice()) };

            if !case::is_camel_case(name) {
                let suggestion = case::to_camel_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("method name should use `camelCase`"),
                    source.clone(),
                    method.name.initial_position(),
                    method.name.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source,
                    method.initial_position(),
                    method.final_position(),
                ))
                .with_note(format!("try renaming the method to `{}`", suggestion));

                return vec![issue];
            }
        } else if let Some(method) = downcast::<ConcreteMethodDefinition>(node) {
            // SAFETY: We know that the class name is valid UTF-8 because the parser
            let name = unsafe { std::str::from_utf8_unchecked(method.name.value.bytes.as_slice()) };

            if !case::is_camel_case(name) {
                let suggestion = case::to_camel_case(name);

                let issue = Issue::note(
                    AnalyzerIssueCode::NamingConventionViolation,
                    format!("method name should use `camelCase`"),
                    source.clone(),
                    method.name.initial_position(),
                    method.name.final_position(),
                )
                .with_annotation(Annotation::secondary(
                    source,
                    method.initial_position(),
                    method.final_position(),
                ))
                .with_note(format!("try renaming the method to `{}`", suggestion));

                return vec![issue];
            }
        }

        return vec![];
    }
}
