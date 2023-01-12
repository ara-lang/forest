use ara_reporting::issue::{Issue, IssueSeverity};
use rustc_hash::FxHashMap;

use ara_parser::tree::TreeMap;
use ara_reporting::{Report, ReportFooter};
use ara_source::SourceMap;

use crate::analyzer::code_info::definition_reference_storage::DefinitionReferenceStorage;
use crate::analyzer::visitor::assign_to_this::AssignToThis;
use crate::analyzer::visitor::assign_to_unwriteable_expression::AssignToUnwriteableExpression;
use crate::analyzer::visitor::await_in_loop::AwaitInLoop;
use crate::analyzer::visitor::builtin_types_generic_arguments_count::BuiltinTypesGenericArgumentsCount;
use crate::analyzer::visitor::default_for_variadic::DefaultForVariadic;
use crate::analyzer::visitor::definition_collector::DefinitionCollector;
use crate::analyzer::visitor::discard_operation::DiscardOperation;
use crate::analyzer::visitor::duplicate_parameter::DuplicateParameter;
use crate::analyzer::visitor::invalid_operand_for_arithmetic_operation::InvalidArthmeticOperation;
use crate::analyzer::visitor::naming_convention::NamingConvention;
use crate::analyzer::visitor::operation_cannot_be_used_for_reading::OperationCannotBeUsedForReading;
use crate::analyzer::visitor::parameters_after_variadic::ParametersAfterVariadic;
use crate::analyzer::visitor::redundant_import::RedundantImport;
use crate::analyzer::visitor::required_parameter_after_optional::RequiredParameterAfterOptional;
use crate::analyzer::visitor::return_from_constructor::ReturnFromConstructor;
use crate::analyzer::visitor::return_from_never_function::ReturnFromNeverFunction;
use crate::analyzer::visitor::return_from_void_function::ReturnFromVoidFunction;
use crate::analyzer::visitor::standalone_block_statement::StandaloneBlockStatement;
use crate::analyzer::visitor::ternary_operation_should_be_an_if_statement::TernaryOperationShouldBeAnIfStatement;
use crate::analyzer::visitor::unreachable_code::UnreachableCode;
use crate::analyzer::visitor::unsafe_finally_block::UnsafeFinallyBlock;
use crate::analyzer::visitor::using_this_outside_of_class_scope::UsingThisOutsideOfClassContext;
use crate::config::Configuration;
use crate::error::Result;

pub mod code_info;
pub mod issue;
pub mod traverser;
pub mod visitor;

pub struct AnalysisReport {
    pub report: Option<Report>,
    pub definitions_storage: DefinitionReferenceStorage,
}

pub struct Analyzer<'a> {
    pub config: &'a Configuration,
}

impl<'a> Analyzer<'a> {
    pub fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    pub fn analyze(&self, source_map: &SourceMap, tree_map: &TreeMap) -> Result<AnalysisReport> {
        let mut collector = DefinitionCollector {
            definitions: FxHashMap::default(),
        };

        let mut issues = traverser::traverse(
            &source_map,
            &tree_map,
            vec![
                Box::new(&mut NamingConvention::new()),
                Box::new(&mut RequiredParameterAfterOptional::new()),
                Box::new(&mut AwaitInLoop::new()),
                Box::new(&mut DiscardOperation::new()),
                Box::new(&mut TernaryOperationShouldBeAnIfStatement::new()),
                Box::new(&mut OperationCannotBeUsedForReading::new()),
                Box::new(&mut UnreachableCode::new()),
                Box::new(&mut InvalidArthmeticOperation::new()),
                Box::new(&mut RedundantImport::new()),
                Box::new(&mut ReturnFromConstructor::new()),
                Box::new(&mut AssignToThis::new()),
                Box::new(&mut AssignToUnwriteableExpression::new()),
                Box::new(&mut StandaloneBlockStatement::new()),
                Box::new(&mut UsingThisOutsideOfClassContext::new()),
                Box::new(&mut UnsafeFinallyBlock::new()),
                Box::new(&mut BuiltinTypesGenericArgumentsCount::new()),
                Box::new(&mut ParametersAfterVariadic::new()),
                Box::new(&mut DefaultForVariadic::new()),
                Box::new(&mut ReturnFromVoidFunction::new()),
                Box::new(&mut ReturnFromNeverFunction::new()),
                Box::new(&mut DuplicateParameter::new()),
                Box::new(&mut collector),
            ],
        )?;

        let (definitions_storage, mut definitions_issues) =
            code_info::definition_reference_collector::collect_definitions(&collector.definitions);

        issues.append(&mut definitions_issues);

        let report = build_report(&self.config, issues);
        let should_continue = if let Some(report) = &report {
            report.severity().unwrap() < IssueSeverity::Error
        } else {
            true
        };

        if !should_continue {
            return Ok(AnalysisReport {
                report,
                definitions_storage,
            });
        }

        todo!();
    }
}

/// Build a report from the given issues.
///
/// If no issues are given, `None` is returned.
fn build_report(config: &Configuration, issues: Vec<Issue>) -> Option<Report> {
    let mut attempted_to_ignore_errors = 0;
    let mut ignored_issues = 0;
    let issues = {
        let mut result = vec![];
        for issue in issues {
            if config.analyzer.ignore.contains(&issue.code) {
                if issue.severity >= IssueSeverity::Error {
                    result.push(issue);

                    attempted_to_ignore_errors += 1;
                } else {
                    ignored_issues += 1;
                }
            } else {
                result.push(issue);
            }
        }

        result
    };

    if issues.is_empty() {
        None
    } else {
        let mut footer =
            ReportFooter::new("failed to analyze the project due to the issue(s) above");

        let mut severities = issues
            .iter()
            .map(|issue| issue.severity)
            .collect::<Vec<IssueSeverity>>();

        severities.sort();

        let mut severities_count: Vec<(&IssueSeverity, usize)> = Vec::new();
        for severity in &severities {
            if let Some(_) = severities_count.iter().find(|(s, _)| *s == severity) {
                continue;
            }

            let count = severities.iter().filter(|s| *s == severity).count();
            severities_count.push((severity, count));
        }

        footer.notes.push(format!(
            "summary: {}",
            severities_count
                .iter()
                .map(|(severity, count)| format!("{} {}(s)", count, severity))
                .collect::<Vec<String>>()
                .join(", ")
        ));

        if ignored_issues > 0 {
            footer
                .notes
                .push(format!("{} issue(s) were ignored", ignored_issues));
        }

        if attempted_to_ignore_errors > 0 {
            footer.notes.push(format!(
                "{} issue(s) were attempted to be ignored but were errors",
                attempted_to_ignore_errors
            ));
        }

        Some(Report {
            issues,
            footer: Some(footer),
        })
    }
}
