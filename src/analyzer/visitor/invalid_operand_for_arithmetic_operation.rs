use ara_parser::tree::downcast;
use ara_parser::tree::expression::literal::Literal;
use ara_parser::tree::expression::magic_constant::MagicConstant;
use ara_parser::tree::expression::operator::ArithmeticOperationExpression;
use ara_parser::tree::expression::operator::ArrayOperationExpression;
use ara_parser::tree::expression::operator::AsyncOperationExpression;
use ara_parser::tree::expression::operator::ClassOperationExpression;
use ara_parser::tree::expression::operator::CoalesceOperationExpression;
use ara_parser::tree::expression::operator::FunctionOperationExpression;
use ara_parser::tree::expression::operator::ObjectOperationExpression;
use ara_parser::tree::expression::operator::TernaryOperationExpression;
use ara_parser::tree::expression::operator::TypeOperationExpression;
use ara_parser::tree::expression::Expression;
use ara_parser::tree::Node;
use ara_reporting::annotation::Annotation;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct InvalidArthmeticOperation;

impl InvalidArthmeticOperation {
    pub fn new() -> Self {
        Self
    }

    fn get_invalid_operand(
        source: &str,
        expression: &Expression,
        allow_string: bool,
        allow_parenthesized: bool,
        allow_null: bool,
    ) -> Option<Annotation> {
        match &expression {
            Expression::Parenthesized(expression) => {
                if allow_parenthesized {
                    Self::get_invalid_operand(
                        source,
                        &expression.expression,
                        allow_string,
                        allow_parenthesized,
                        allow_null,
                    )
                } else {
                    Some(Annotation::secondary(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    ))
                }
            }
            Expression::ExitConstruct(expression) => Some(Annotation::secondary(
                source,
                expression.initial_position(),
                expression.final_position(),
            )),
            Expression::Literal(literal) => match &literal {
                Literal::True(_) | Literal::False(_) => Some(Annotation::secondary(
                    source,
                    literal.initial_position(),
                    literal.final_position(),
                )),
                Literal::String(_) => {
                    if !allow_string {
                        Some(Annotation::secondary(
                            source,
                            literal.initial_position(),
                            literal.final_position(),
                        ))
                    } else {
                        None
                    }
                }
                Literal::Null(_) => {
                    if !allow_null {
                        Some(Annotation::secondary(
                            source,
                            literal.initial_position(),
                            literal.final_position(),
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Expression::ArithmeticOperation(_) => {
                // nested arithmetic operations will be visited by the traverser again
                None
            }
            Expression::AsyncOperation(operation) => match &operation {
                AsyncOperationExpression::Async { .. }
                | AsyncOperationExpression::Concurrently { .. } => Some(Annotation::secondary(
                    source,
                    operation.initial_position(),
                    operation.final_position(),
                )),
                _ => None,
            },
            Expression::ArrayOperation(operation) => match &operation {
                ArrayOperationExpression::Push { .. }
                | ArrayOperationExpression::Unset { .. }
                | ArrayOperationExpression::Isset { .. }
                | ArrayOperationExpression::In { .. } => Some(Annotation::secondary(
                    source,
                    operation.initial_position(),
                    operation.final_position(),
                )),
                _ => None,
            },
            // assignment operations cannot be used for reading, they are handled by the OperationCannotBeUsedForReading visitor
            Expression::AssignmentOperation(_) => None,
            Expression::ClassOperation(operation) => match &operation {
                ClassOperationExpression::Initialization { .. }
                | ClassOperationExpression::AnonymousInitialization { .. }
                | ClassOperationExpression::StaticMethodClosureCreation { .. } => {
                    Some(Annotation::secondary(
                        source,
                        operation.initial_position(),
                        operation.final_position(),
                    ))
                }
                ClassOperationExpression::ConstantFetch { constant, .. } => {
                    if constant.value.to_string() == "class" {
                        Some(Annotation::secondary(
                            source,
                            operation.initial_position(),
                            operation.final_position(),
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Expression::CoalesceOperation(CoalesceOperationExpression::Coalesce {
                left,
                right,
                ..
            }) => {
                let left = Self::get_invalid_operand(
                    source,
                    left,
                    allow_parenthesized,
                    allow_string,
                    true,
                );
                if let Some(left) = left {
                    Some(left)
                } else {
                    Self::get_invalid_operand(
                        source,
                        right,
                        allow_parenthesized,
                        allow_string,
                        allow_null,
                    )
                }
            }
            Expression::FunctionOperation(operation) => match &operation {
                FunctionOperationExpression::ClosureCreation { .. } => Some(Annotation::secondary(
                    source,
                    operation.initial_position(),
                    operation.final_position(),
                )),
                _ => None,
            },
            Expression::LogicalOperation(_) => Some(Annotation::secondary(
                source,
                expression.initial_position(),
                expression.final_position(),
            )),
            Expression::ObjectOperation(operation) => match &operation {
                ObjectOperationExpression::Clone { .. }
                | ObjectOperationExpression::NullsafeMethodCall { .. }
                | ObjectOperationExpression::MethodClosureCreation { .. }
                | ObjectOperationExpression::NullsafePropertyFetch { .. } => {
                    Some(Annotation::secondary(
                        source,
                        operation.initial_position(),
                        operation.final_position(),
                    ))
                }
                _ => None,
            },
            Expression::TernaryOperation(operation) => match &operation {
                TernaryOperationExpression::Ternary {
                    if_true, if_false, ..
                } => {
                    let if_true = Self::get_invalid_operand(
                        source,
                        if_true,
                        allow_string,
                        allow_parenthesized,
                        true,
                    );
                    if let Some(if_true) = if_true {
                        Some(if_true)
                    } else {
                        Self::get_invalid_operand(
                            source,
                            if_false,
                            allow_string,
                            allow_parenthesized,
                            allow_null,
                        )
                    }
                }
                TernaryOperationExpression::ImplicitShortTernary { .. }
                | TernaryOperationExpression::ShortTernary { .. } => {
                    // short ternary operations will always result in a boolean, so they are invalid operands
                    Some(Annotation::secondary(
                        source,
                        operation.initial_position(),
                        operation.final_position(),
                    ))
                }
            },
            Expression::Variable(variable) => {
                if variable.name.to_string() == "$this" {
                    Some(Annotation::secondary(
                        source,
                        variable.initial_position(),
                        variable.final_position(),
                    ))
                } else {
                    None
                }
            }
            Expression::TypeOperation(operation) => match &operation {
                TypeOperationExpression::Instanceof { .. } | TypeOperationExpression::Is { .. } => {
                    Some(Annotation::secondary(
                        source,
                        operation.initial_position(),
                        operation.final_position(),
                    ))
                }
                _ => None,
            },
            Expression::ComparisonOperation(_)
            | Expression::RangeOperation(_)
            | Expression::ExceptionOperation(_)
            | Expression::StringOperation(_)
            | Expression::AnonymousFunction(_)
            | Expression::ArrowFunction(_)
            | Expression::Vec(_)
            | Expression::Dict(_)
            | Expression::Tuple(_) => Some(Annotation::secondary(
                source,
                expression.initial_position(),
                expression.final_position(),
            )),
            Expression::MagicConstant(constant) => match &constant {
                MagicConstant::Line { .. } => None,
                _ => {
                    if allow_string {
                        None
                    } else {
                        Some(Annotation::secondary(
                            source,
                            constant.initial_position(),
                            constant.final_position(),
                        ))
                    }
                }
            },
            _ => None,
        }
    }
}

impl Visitor for InvalidArthmeticOperation {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        if let Some(operation) = downcast::<ArithmeticOperationExpression>(node) {
            match &operation {
                ArithmeticOperationExpression::Addition {
                    left,
                    plus: operation,
                    right,
                    ..
                }
                | ArithmeticOperationExpression::Subtraction {
                    left,
                    minus: operation,
                    right,
                    ..
                }
                | ArithmeticOperationExpression::Multiplication {
                    left,
                    asterisk: operation,
                    right,
                    ..
                }
                | ArithmeticOperationExpression::Division {
                    left,
                    slash: operation,
                    right,
                    ..
                }
                | ArithmeticOperationExpression::Modulo {
                    left,
                    percent: operation,
                    right,
                    ..
                }
                | ArithmeticOperationExpression::Exponentiation {
                    left,
                    pow: operation,
                    right,
                    ..
                } => {
                    let left_invalid_operand =
                        Self::get_invalid_operand(source, left, false, true, false);
                    let right_invalid_operand =
                        Self::get_invalid_operand(source, right, false, true, false);

                    if left_invalid_operand.is_some() || right_invalid_operand.is_some() {
                        let mut issue = Issue::error(
                            AnalyzerIssueCode::InvalidOperandForArithmeticOperation,
                            "invalid operand for arithmetic operation",
                            source,
                            *operation,
                            operation + 1,
                        );

                        if let Some(left_invalid_operand) = left_invalid_operand {
                            issue = issue.with_annotation(
                                left_invalid_operand.with_message("invalid left operand"),
                            );
                        }

                        if let Some(right_invalid_operand) = right_invalid_operand {
                            issue = issue.with_annotation(
                                right_invalid_operand.with_message("invalid right operand"),
                            );
                        }

                        return vec![issue];
                    }
                }
                ArithmeticOperationExpression::Negative {
                    minus: operation,
                    right: expression,
                    ..
                }
                | ArithmeticOperationExpression::Positive {
                    plus: operation,
                    right: expression,
                    ..
                } => {
                    let right_invalid_operand =
                        Self::get_invalid_operand(source, expression, false, false, false);

                    if let Some(right_invalid_operand) = right_invalid_operand {
                        let issue = Issue::error(
                            AnalyzerIssueCode::InvalidOperandForArithmeticOperation,
                            "invalid operand for arithmetic operation",
                            source,
                            *operation,
                            operation + 1,
                        )
                        .with_annotation(right_invalid_operand.with_message("invalid operand"));

                        return vec![issue];
                    }
                }
                ArithmeticOperationExpression::PreIncrement {
                    increment: operation,
                    right: expression,
                    ..
                }
                | ArithmeticOperationExpression::PreDecrement {
                    decrement: operation,
                    right: expression,
                    ..
                }
                | ArithmeticOperationExpression::PostIncrement {
                    left: expression,
                    increment: operation,
                    ..
                }
                | ArithmeticOperationExpression::PostDecrement {
                    left: expression,
                    decrement: operation,
                    ..
                } => {
                    let right_invalid_operand =
                        Self::get_invalid_operand(source, expression, true, false, false);

                    if let Some(right_invalid_operand) = right_invalid_operand {
                        let issue = Issue::error(
                            AnalyzerIssueCode::InvalidOperandForArithmeticOperation,
                            "invalid operand for arithmetic operation",
                            source,
                            *operation,
                            operation + 2,
                        )
                        .with_annotation(right_invalid_operand.with_message("invalid operand"));

                        return vec![issue];
                    }
                }
            }
        }

        vec![]
    }
}
