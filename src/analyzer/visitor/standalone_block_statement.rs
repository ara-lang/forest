use ara_parser::tree::downcast;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct StandaloneBlockStatement;

impl StandaloneBlockStatement {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for StandaloneBlockStatement {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        if let Some(Statement::Block(statement)) = downcast::<Statement>(node) {
            let issue = Issue::error(
                AnalyzerIssueCode::CannotUseStandaloneBlockStatement,
                "cannot use a standalone block statement",
            )
            .with_source(
                source,
                statement.initial_position(),
                statement.final_position(),
            )
            .with_note("remove the outter brackets");

            return vec![issue];
        }

        vec![]
    }
}
