use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

pub mod assign_to_this;
pub mod assign_to_unwriteable_expression;
pub mod await_in_loop;
pub mod builtin_types_generic_arguments_count;
pub mod default_for_variadic;
pub mod definition_collector;
pub mod discard_operation;
pub mod duplicate_parameter;
pub mod invalid_operand_for_arithmetic_operation;
pub mod naming_convention;
pub mod operation_cannot_be_used_for_reading;
pub mod parameters_after_variadic;
pub mod required_parameter_after_optional;
pub mod return_from_constructor;
pub mod return_from_never_function;
pub mod return_from_void_function;
pub mod standalone_block_statement;
pub mod ternary_operation_should_be_an_if_statement;
pub mod unreachable_code;
pub mod unsafe_finally_block;
pub mod using_this_outside_of_class_scope;

pub trait Visitor {
    fn visit_node<'a>(
        &mut self,
        source: &str,
        node: &'a dyn Node,
        ancestry: &mut Vec<&'a dyn Node>,
    ) -> Vec<Issue> {
        let mut issues = vec![];

        issues.append(&mut self.visit(source, node, &ancestry));

        ancestry.push(node);
        for child in node.children() {
            issues.append(&mut self.visit_node(source, child, ancestry));
        }
        ancestry.pop();

        issues
    }

    fn visit(&mut self, source: &str, node: &dyn Node, ancestry: &Vec<&dyn Node>) -> Vec<Issue>;
}
