use ara_parser::tree::downcast;
use ara_parser::tree::expression::argument::ArgumentExpression;
use ara_parser::tree::expression::construct::ExitConstructExpression;
use ara_parser::tree::expression::control_flow::MatchArmConditionExpression;
use ara_parser::tree::expression::operator::{
    ArithmeticOperationExpression, ArrayOperationExpression, AssignmentOperationExpression,
    AsyncOperationExpression, BitwiseOperationExpression, ClassOperationExpression,
    CoalesceOperationExpression, ComparisonOperationExpression, ExceptionOperationExpression,
    FunctionOperationExpression, GeneratorOperationExpression, LogicalOperationExpression,
    ObjectOperationExpression, RangeOperationExpression, StringOperationExpression,
    TernaryOperationExpression, TypeOperationExpression,
};
use ara_parser::tree::expression::Expression;
use ara_parser::tree::statement::r#loop::{ForIteratorStatement, ForeachIteratorStatement};
use ara_parser::tree::statement::r#return::ReturnStatement;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

pub struct OperationCannotBeUsedForReading;

impl OperationCannotBeUsedForReading {
    pub fn new() -> Self {
        Self
    }

    fn analyze_expression(source: &str, expression: &Expression, discarded: bool) -> Vec<Issue> {
        match &expression {
            Expression::Parenthesized(expression) => {
                Self::analyze_expression(source, &expression.expression, discarded)
            }
            Expression::ArrayOperation(operation) => match operation {
                ArrayOperationExpression::Push { .. } => vec![Issue::error(
                    AnalyzerIssueCode::ArrayPushOperationCannotBeUsedForReading,
                    "array push operation cannot be used for reading",
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )],
                ArrayOperationExpression::Unset { item, .. }
                | ArrayOperationExpression::Isset { item, .. } => {
                    Self::analyze_expression(source, &item, discarded)
                }
                ArrayOperationExpression::In { item, array, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &item, discarded));
                    issues.append(&mut Self::analyze_expression(source, &array, discarded));

                    issues
                }
                _ => vec![],
            },
            Expression::AsyncOperation(operation) => match &operation {
                AsyncOperationExpression::Async { expression, .. }
                | AsyncOperationExpression::Await { expression, .. } => {
                    Self::analyze_expression(source, &expression, discarded)
                }
                AsyncOperationExpression::Concurrently { expressions, .. } => {
                    let mut issues = vec![];
                    for expression in &expressions.inner {
                        issues.append(&mut Self::analyze_expression(
                            source,
                            &expression,
                            discarded,
                        ));
                    }

                    issues
                }
            },
            Expression::AssignmentOperation(assignment) => match &assignment {
                AssignmentOperationExpression::Assignment { left, right, .. } => {
                    let mut issues = vec![];

                    if !discarded {
                        issues.push(Issue::error(
                            AnalyzerIssueCode::AssignmentOperationCannotBeUsedForReading,
                            "assignment operation cannot be used for reading",
                            source,
                            expression.initial_position(),
                            expression.final_position(),
                        ));
                    }

                    issues.append(&mut Self::analyze_expression(source, &left, false));
                    issues.append(&mut Self::analyze_expression(source, &right, false));

                    issues
                }
                AssignmentOperationExpression::Addition { left, right, .. }
                | AssignmentOperationExpression::Subtraction { left, right, .. }
                | AssignmentOperationExpression::Multiplication { left, right, .. }
                | AssignmentOperationExpression::Division { left, right, .. }
                | AssignmentOperationExpression::Modulo { left, right, .. }
                | AssignmentOperationExpression::Exponentiation { left, right, .. }
                | AssignmentOperationExpression::Concat { left, right, .. }
                | AssignmentOperationExpression::BitwiseAnd { left, right, .. }
                | AssignmentOperationExpression::BitwiseOr { left, right, .. }
                | AssignmentOperationExpression::BitwiseXor { left, right, .. }
                | AssignmentOperationExpression::LeftShift { left, right, .. }
                | AssignmentOperationExpression::RightShift { left, right, .. }
                | AssignmentOperationExpression::Coalesce { left, right, .. } => {
                    let mut issues = vec![];

                    if !discarded {
                        issues.push(Issue::error(
                            AnalyzerIssueCode::AssignmentOperationCannotBeUsedForReading,
                            "assignment operation cannot be used for reading",
                            source,
                            expression.initial_position(),
                            expression.final_position(),
                        ));
                    }

                    issues.append(&mut Self::analyze_expression(source, &left, false));
                    issues.append(&mut Self::analyze_expression(source, &right, false));

                    issues
                }
            },
            Expression::BitwiseOperation(operation) => match &operation {
                BitwiseOperationExpression::And { left, right, .. }
                | BitwiseOperationExpression::Or { left, right, .. }
                | BitwiseOperationExpression::Xor { left, right, .. }
                | BitwiseOperationExpression::LeftShift { left, right, .. }
                | BitwiseOperationExpression::RightShift { left, right, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &left, discarded));
                    issues.append(&mut Self::analyze_expression(source, &right, discarded));

                    issues
                }
                BitwiseOperationExpression::Not { right, .. } => {
                    Self::analyze_expression(source, &right, discarded)
                }
            },
            Expression::ClassOperation(operation) => match &operation {
                ClassOperationExpression::StaticMethodCall { class, .. }
                | ClassOperationExpression::StaticMethodClosureCreation { class, .. }
                | ClassOperationExpression::StaticPropertyFetch { class, .. }
                | ClassOperationExpression::ConstantFetch { class, .. } => {
                    Self::analyze_expression(source, &class, discarded)
                }
                _ => vec![],
            },
            Expression::FunctionOperation(operation) => match &operation {
                FunctionOperationExpression::Call { function, .. }
                | FunctionOperationExpression::ClosureCreation { function, .. } => {
                    Self::analyze_expression(source, &function, discarded)
                }
            },
            Expression::CoalesceOperation(operation) => match &operation {
                CoalesceOperationExpression::Coalesce { left, right, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &left, discarded));
                    issues.append(&mut Self::analyze_expression(source, &right, discarded));

                    issues
                }
            },
            Expression::ComparisonOperation(operation) => match &operation {
                ComparisonOperationExpression::Equal { left, right, .. }
                | ComparisonOperationExpression::NotEqual { left, right, .. }
                | ComparisonOperationExpression::Identical { left, right, .. }
                | ComparisonOperationExpression::NotIdentical { left, right, .. }
                | ComparisonOperationExpression::LessThan { left, right, .. }
                | ComparisonOperationExpression::LessThanOrEqual { left, right, .. }
                | ComparisonOperationExpression::GreaterThan { left, right, .. }
                | ComparisonOperationExpression::GreaterThanOrEqual { left, right, .. }
                | ComparisonOperationExpression::Spaceship { left, right, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &left, discarded));
                    issues.append(&mut Self::analyze_expression(source, &right, discarded));

                    issues
                }
            },
            Expression::ExceptionOperation(operation) => match &operation {
                ExceptionOperationExpression::Throw { value, .. } => {
                    let mut issues = vec![];
                    if !discarded {
                        issues.push(Issue::error(
                            AnalyzerIssueCode::ThrowExpressionCannotBeUsedForReading,
                            "throw expression cannot be used for reading",
                            source,
                            expression.initial_position(),
                            expression.final_position(),
                        ));
                    }

                    issues.append(&mut Self::analyze_expression(source, &value, discarded));

                    issues
                }
            },
            Expression::GeneratorOperation(operation) => match &operation {
                GeneratorOperationExpression::YieldValue { value, .. }
                | GeneratorOperationExpression::YieldFrom { value, .. } => {
                    Self::analyze_expression(source, &value, discarded)
                }
                GeneratorOperationExpression::YieldKeyValue { key, value, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &key, discarded));
                    issues.append(&mut Self::analyze_expression(source, &value, discarded));

                    issues
                }
                _ => vec![],
            },
            Expression::LogicalOperation(operation) => match &operation {
                LogicalOperationExpression::And { left, right, .. }
                | LogicalOperationExpression::Or { left, right, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &left, discarded));
                    issues.append(&mut Self::analyze_expression(source, &right, discarded));

                    issues
                }
                LogicalOperationExpression::Not { right, .. } => {
                    Self::analyze_expression(source, &right, discarded)
                }
            },
            Expression::ObjectOperation(operation) => match &operation {
                ObjectOperationExpression::PropertyFetch { object, .. }
                | ObjectOperationExpression::NullsafePropertyFetch { object, .. }
                | ObjectOperationExpression::MethodCall { object, .. }
                | ObjectOperationExpression::NullsafeMethodCall { object, .. }
                | ObjectOperationExpression::MethodClosureCreation { object, .. } => {
                    Self::analyze_expression(source, &object, discarded)
                }
                _ => vec![],
            },
            Expression::RangeOperation(operation) => match &operation {
                RangeOperationExpression::Between { from, to, .. }
                | RangeOperationExpression::BetweenInclusive { from, to, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &from, discarded));
                    issues.append(&mut Self::analyze_expression(source, &to, discarded));

                    issues
                }
                RangeOperationExpression::To { to, .. }
                | RangeOperationExpression::ToInclusive { to, .. } => {
                    Self::analyze_expression(source, &to, discarded)
                }
                RangeOperationExpression::From { from, .. } => {
                    Self::analyze_expression(source, &from, discarded)
                }
                _ => vec![],
            },
            Expression::StringOperation(operation) => match &operation {
                StringOperationExpression::Concat { left, right, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &left, discarded));
                    issues.append(&mut Self::analyze_expression(source, &right, discarded));

                    issues
                }
            },
            Expression::TypeOperation(operation) => match &operation {
                TypeOperationExpression::Is { left, .. }
                | TypeOperationExpression::As { left, .. }
                | TypeOperationExpression::Instanceof { left, .. }
                | TypeOperationExpression::Into { left, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &left, discarded));

                    issues
                }
            },
            Expression::TernaryOperation(operation) => match &operation {
                TernaryOperationExpression::Ternary {
                    condition,
                    if_true,
                    if_false,
                    ..
                } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &condition, discarded));
                    issues.append(&mut Self::analyze_expression(source, &if_true, discarded));
                    issues.append(&mut Self::analyze_expression(source, &if_false, discarded));

                    issues
                }
                TernaryOperationExpression::ShortTernary {
                    condition,
                    if_false,
                    ..
                }
                | TernaryOperationExpression::ImplicitShortTernary {
                    condition,
                    if_false,
                    ..
                } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &condition, discarded));
                    issues.append(&mut Self::analyze_expression(source, &if_false, discarded));

                    issues
                }
            },
            Expression::Match(expression) => {
                let mut issues = vec![];
                issues.append(&mut Self::analyze_expression(
                    source,
                    &expression.expression,
                    discarded,
                ));

                for arm in &expression.body.arms.inner {
                    match &arm.condition {
                        MatchArmConditionExpression::Expressions(expressions) => {
                            for expression in &expressions.inner {
                                issues.append(&mut Self::analyze_expression(
                                    source,
                                    &expression,
                                    discarded,
                                ));
                            }
                        }
                        _ => {}
                    }
                    issues.append(&mut Self::analyze_expression(
                        source,
                        &arm.expression,
                        discarded,
                    ));
                }

                issues
            }
            Expression::Vec(vector) => {
                let mut issues = vec![];
                for element in &vector.elements.inner {
                    issues.append(&mut Self::analyze_expression(
                        source,
                        &element.value,
                        discarded,
                    ));
                }

                issues
            }
            Expression::Dict(dictionary) => {
                let mut issues = vec![];
                for element in &dictionary.elements.inner {
                    issues.append(&mut Self::analyze_expression(
                        source,
                        &element.key,
                        discarded,
                    ));
                    issues.append(&mut Self::analyze_expression(
                        source,
                        &element.value,
                        discarded,
                    ));
                }

                issues
            }
            Expression::Tuple(tuple) => {
                let mut issues = vec![];
                for element in &tuple.elements.inner {
                    issues.append(&mut Self::analyze_expression(source, &element, discarded));
                }

                issues
            }
            Expression::ExitConstruct(construct) => {
                let mut issues = vec![];

                if !discarded {
                    issues.push(Issue::error(
                        AnalyzerIssueCode::ExitExpressionCannotBeUsedForReading,
                        "exit expression cannot be used for reading",
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    ));
                }

                match &construct {
                    ExitConstructExpression::ExitWith { value, .. } => {
                        if let Some(value) = value {
                            issues.append(&mut Self::analyze_expression(source, &value, discarded));
                        }
                    }
                    _ => {}
                }

                issues
            }
            Expression::ArithmeticOperation(operation) => match &operation {
                ArithmeticOperationExpression::Addition { left, right, .. }
                | ArithmeticOperationExpression::Subtraction { left, right, .. }
                | ArithmeticOperationExpression::Multiplication { left, right, .. }
                | ArithmeticOperationExpression::Division { left, right, .. }
                | ArithmeticOperationExpression::Modulo { left, right, .. }
                | ArithmeticOperationExpression::Exponentiation { left, right, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, &left, discarded));
                    issues.append(&mut Self::analyze_expression(source, &right, discarded));

                    issues
                }
                ArithmeticOperationExpression::Negative { right, .. }
                | ArithmeticOperationExpression::Positive { right, .. }
                | ArithmeticOperationExpression::PreIncrement { right, .. }
                | ArithmeticOperationExpression::PreDecrement { right, .. } => {
                    Self::analyze_expression(source, &right, discarded)
                }
                ArithmeticOperationExpression::PostIncrement { left, .. }
                | ArithmeticOperationExpression::PostDecrement { left, .. } => {
                    Self::analyze_expression(source, &left, discarded)
                }
            },
            _ => vec![],
        }
    }
}

impl Visitor for OperationCannotBeUsedForReading {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &Vec<&dyn Node>) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(statement) = downcast::<Statement>(node) {
            match &statement {
                Statement::DoWhile(do_while) => {
                    issues.append(&mut Self::analyze_expression(
                        source,
                        &do_while.condition,
                        false,
                    ));
                }
                Statement::While(r#while) => {
                    issues.append(&mut Self::analyze_expression(
                        source,
                        &r#while.condition,
                        false,
                    ));
                }
                Statement::For(r#for) => match &r#for.iterator {
                    ForIteratorStatement::Standalone {
                        initializations,
                        conditions,
                        r#loop,
                        ..
                    }
                    | ForIteratorStatement::Parenthesized {
                        initializations,
                        conditions,
                        r#loop,
                        ..
                    } => {
                        for initialization in &initializations.inner {
                            issues.append(&mut Self::analyze_expression(
                                source,
                                &initialization,
                                true,
                            ));
                        }
                        for condition in &conditions.inner {
                            issues.append(&mut Self::analyze_expression(source, &condition, false));
                        }
                        for loop_ in &r#loop.inner {
                            issues.append(&mut Self::analyze_expression(source, &loop_, true));
                        }
                    }
                },
                Statement::Foreach(foreach) => match &foreach.iterator {
                    ForeachIteratorStatement::Value { expression, .. }
                    | ForeachIteratorStatement::ParenthesizedValue { expression, .. }
                    | ForeachIteratorStatement::KeyAndValue { expression, .. }
                    | ForeachIteratorStatement::ParenthesizedKeyAndValue { expression, .. } => {
                        issues.append(&mut Self::analyze_expression(source, &expression, false));
                    }
                },
                Statement::If(r#if) => {
                    issues.append(&mut Self::analyze_expression(
                        source,
                        &r#if.condition,
                        false,
                    ));
                }
                Statement::Using(using) => {
                    for assignment in &using.assignments.inner {
                        issues.append(&mut Self::analyze_expression(
                            source,
                            &assignment.expression,
                            false,
                        ));
                    }
                }
                Statement::Expression(expression) => {
                    for issue in Self::analyze_expression(source, &expression.expression, true) {
                        issues.push(issue);
                    }
                }
                Statement::Return(r#return) => match r#return.as_ref() {
                    ReturnStatement::Explicit { expression, .. } => {
                        if let Some(expression) = expression {
                            issues.append(&mut Self::analyze_expression(
                                source,
                                &expression,
                                false,
                            ));
                        }
                    }
                    ReturnStatement::Implicit { expression, .. } => {
                        issues.append(&mut Self::analyze_expression(source, &expression, false));
                    }
                },
                _ => {}
            }
        } else if let Some(argument) = downcast::<ArgumentExpression>(node) {
            match &argument {
                ArgumentExpression::Value { value, .. }
                | ArgumentExpression::Spread { value, .. }
                | ArgumentExpression::ReverseSpread { value, .. }
                | ArgumentExpression::Named { value, .. } => {
                    for issue in Self::analyze_expression(source, &value, false) {
                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }
}
