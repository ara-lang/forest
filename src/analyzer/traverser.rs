use ara_parser::tree::TreeMap;
use ara_reporting::issue::{Issue, IssueSeverity};
use ara_source::source::SourceKind;
use ara_source::SourceMap;

use crate::analyzer::visitor::Visitor;
use crate::error::Result;

pub fn traverse(
    source_map: &SourceMap,
    tree_map: &TreeMap,
    mut visitors: Vec<Box<&mut dyn Visitor>>,
) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    for tree in &tree_map.trees {
        for visitor in &mut visitors {
            let source_kind = source_map.named(tree.source.to_string()).unwrap().kind;

            let mut ancestry = vec![];
            for issue in visitor.visit_node(&tree.source, &tree.definitions, &mut ancestry) {
                if source_kind != SourceKind::Definition || issue.severity > IssueSeverity::Warning
                {
                    issues.push(issue);
                }
            }

            assert!(
                ancestry.is_empty(),
                "node(s) left in ancestry stack after visiting children"
            );
        }
    }

    Ok(issues)
}
