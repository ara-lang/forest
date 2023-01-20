use ara_parser::tree::definition::r#type::TypeDefinition;
use ara_parser::tree::downcast;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct UnsupportedFeaturesAnalyzer;

impl UnsupportedFeaturesAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Visitor for UnsupportedFeaturesAnalyzer {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(statement) = downcast::<Statement>(node) {
            match &statement {
                Statement::Block(_) => {
                    issues.push(
                        Issue::error(
                            AnalyzerIssueCode::StandaloneBlockStatementsAreNotSupported,
                            "standalone block statements are not supported",
                        )
                        .with_source(
                            source,
                            statement.initial_position(),
                            statement.final_position(),
                        )
                        .with_note("help: remove the outter brackets"),
                    );
                }
                Statement::Empty(_) => {
                    issues.push(
                        Issue::error(
                            AnalyzerIssueCode::EmptyStatementsAreNotSupported,
                            "empty statements are not supported",
                        )
                        .with_source(
                            source,
                            statement.initial_position(),
                            statement.final_position(),
                        )
                        .with_note("help: remove this semicolon"),
                    );
                }
                _ => {}
            };
        }

        if let Some(_) = downcast::<TypeDefinition>(node) {
            // TODO: remove support for i128
        }

        issues
    }
}
