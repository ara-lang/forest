use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

pub mod assign_to_this;
pub mod assign_to_unwriteable_expression;
pub mod await_in_loop;
pub mod definition_collector;
pub mod discard_operation;
pub mod invalid_operand_for_arithmetic_operation;
pub mod keyword_must_be_lowercase;
pub mod modifier_group_definition_analyzer;
pub mod naming_convention;
pub mod operation_cannot_be_used_for_reading;
pub mod parameters_analyzer;
pub mod return_statement_analyzer;
pub mod type_definition_analyzer;
pub mod unsupported_features_analyzer;
pub mod ternary_operation_should_be_an_if_statement;
pub mod try_statement_analyzer;
pub mod unreachable_code;
pub mod self_reference_analyzer;

pub trait Visitor {
    fn visit_node<'a>(
        &mut self,
        source: &str,
        node: &'a dyn Node,
        ancestry: &mut Vec<&'a dyn Node>,
    ) -> Vec<Issue> {
        let mut issues = vec![];

        issues.append(&mut self.visit(source, node, ancestry));

        ancestry.push(node);
        for child in node.children() {
            issues.append(&mut self.visit_node(source, child, ancestry));
        }
        ancestry.pop();

        issues
    }

    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &[&dyn Node]) -> Vec<Issue>;
}
