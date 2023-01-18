use rustc_hash::FxHashMap;

use ara_parser::tree::definition::Definition;
use ara_parser::tree::downcast;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct DefinitionCollector {
    pub definitions: FxHashMap<String, Vec<Definition>>,
}

impl Visitor for DefinitionCollector {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        if let Some(definition) = downcast::<Definition>(node) {
            let source = source.to_string();
            let definitions = self.definitions.entry(source).or_default();

            definitions.push(definition.clone());
        }

        vec![]
    }
}
