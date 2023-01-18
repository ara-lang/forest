use ara_parser::tree::downcast;
use ara_parser::tree::token::Keyword;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct KeywordMustBeInLowercase;

impl KeywordMustBeInLowercase {
    pub fn new() -> Self {
        Self
    }
}

impl Visitor for KeywordMustBeInLowercase {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(keyword) = downcast::<Keyword>(node) {
            let name = keyword.value.to_string();
            let lowercase_name = name.to_ascii_lowercase();

            if lowercase_name != name {
                issues.push(
                    Issue::error(
                        AnalyzerIssueCode::KeywordMustBeInLowercase,
                        format!("keyword `{}` must be in lowercase", lowercase_name),
                    )
                    .with_source(
                        source,
                        keyword.initial_position(),
                        keyword.final_position(),
                    ),
                );
            }
        }

        issues
    }
}
