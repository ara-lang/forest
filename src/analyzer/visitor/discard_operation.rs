use ara_parser::tree::downcast;
use ara_parser::tree::expression::operator::{
    ArrayOperationExpression, AsyncOperationExpression, ClassOperationExpression,
    CoalesceOperationExpression, FunctionOperationExpression, ObjectOperationExpression,
    StringOperationExpression,
};
use ara_parser::tree::expression::Expression;
use ara_parser::tree::statement::block::BlockStatement;
use ara_parser::tree::statement::Statement;
use ara_parser::tree::Node;
use ara_reporting::issue::Issue;

use crate::analyzer::issue::AnalyzerIssueCode;
use crate::analyzer::visitor::Visitor;

#[derive(Debug, Default)]
pub struct DiscardOperation;

impl DiscardOperation {
    pub fn new() -> Self {
        Self
    }

    fn analyze_expression(source: &str, expression: &Expression) -> Vec<Issue> {
        match &expression {
            Expression::Parenthesized(expression) => {
                Self::analyze_expression(source, &expression.expression)
            }
            Expression::Literal(_) => {
                vec![
                    Issue::warning(AnalyzerIssueCode::DontDiscardLiteral, "literal discarded")
                        .with_source(
                            source,
                            expression.initial_position(),
                            expression.final_position(),
                        ),
                ]
            }
            Expression::ArithmeticOperation(_) => {
                vec![Issue::warning(
                    AnalyzerIssueCode::DontDiscardArithmeticOperation,
                    "arithmetic operation discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )]
            }
            Expression::AsyncOperation(operation) => {
                if let AsyncOperationExpression::Async { .. } = operation {
                    vec![Issue::error(
                        AnalyzerIssueCode::DontDiscardAsyncOperation,
                        "async operation discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_note("`Psl\\Async\\Awaitable` object must be handled appropriately.")]
                } else {
                    vec![]
                }
            }
            Expression::ArrayOperation(operation) => match operation {
                ArrayOperationExpression::Access { .. } => {
                    vec![Issue::warning(
                        AnalyzerIssueCode::DontDiscardArrayAccessOperation,
                        "array access operation discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
                ArrayOperationExpression::Isset { .. } => {
                    vec![Issue::warning(
                        AnalyzerIssueCode::DontDiscardArrayIssetOperation,
                        "array isset operation discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
                ArrayOperationExpression::In { .. } => {
                    vec![Issue::warning(
                        AnalyzerIssueCode::DontDiscardArrayInOperation,
                        "array in operation discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
                _ => vec![],
            },
            Expression::BitwiseOperation(_) => {
                vec![Issue::warning(
                    AnalyzerIssueCode::DontDiscardBitwiseOperation,
                    "bitwise operation discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )]
            }
            Expression::CoalesceOperation(operation) => match operation {
                CoalesceOperationExpression::Coalesce { left, right, .. } => {
                    let mut issues = vec![];
                    issues.append(&mut Self::analyze_expression(source, left));
                    issues.append(&mut Self::analyze_expression(source, right));

                    issues
                }
            },
            Expression::ComparisonOperation(_) => {
                vec![Issue::warning(
                    AnalyzerIssueCode::DontDiscardComparisonOperation,
                    "comparison operation discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )]
            }

            Expression::LogicalOperation(_) => {
                vec![Issue::warning(
                    AnalyzerIssueCode::DontDiscardLogicalOperation,
                    "logical operation discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )]
            }
            Expression::Variable(_) => {
                vec![
                    Issue::warning(AnalyzerIssueCode::DontDiscardVariable, "variable discarded")
                        .with_source(
                            source,
                            expression.initial_position(),
                            expression.final_position(),
                        ),
                ]
            }
            Expression::Identifier(_) | Expression::MagicConstant(_) => {
                vec![
                    Issue::warning(AnalyzerIssueCode::DontDiscardConstant, "constant discarded")
                        .with_source(
                            source,
                            expression.initial_position(),
                            expression.final_position(),
                        ),
                ]
            }
            Expression::ClassOperation(operation) => match operation {
                ClassOperationExpression::Initialization { .. }
                | ClassOperationExpression::AnonymousInitialization { .. } => {
                    let issue = Issue::warning(
                        AnalyzerIssueCode::DontDiscardClassInitialization,
                        "class initialization discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_note("if you are running the constructor for its side-effects,")
                    .with_note("consider extracting the logic into a separate function.");

                    vec![issue]
                }
                ClassOperationExpression::StaticPropertyFetch { .. } => {
                    vec![Issue::warning(
                        AnalyzerIssueCode::DontDiscardStaticPropertyFetchOperation,
                        "static property fetch discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
                ClassOperationExpression::ConstantFetch { .. } => {
                    vec![Issue::warning(
                        AnalyzerIssueCode::DontDiscardClassConstantFetchOperation,
                        "class constant fetch discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
                ClassOperationExpression::StaticMethodClosureCreation { .. } => {
                    vec![Issue::warning(
                        AnalyzerIssueCode::DontDiscardStaticMethodClosureCreationOperation,
                        "static method closure creation discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )]
                }
                _ => vec![],
            },
            Expression::FunctionOperation(FunctionOperationExpression::ClosureCreation {
                ..
            }) => {
                vec![Issue::warning(
                    AnalyzerIssueCode::DontDiscardFunctionClosureCreationOperation,
                    "function closure creation discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )]
            }
            Expression::ObjectOperation(operation) => match &operation {
                ObjectOperationExpression::Clone { .. } => {
                    let issue = Issue::warning(
                        AnalyzerIssueCode::DontDiscardObjectCloneOperation,
                        "object clone discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_note("if you are triggering the `__clone` method for its side-effects,")
                    .with_note("consider restructuring that method.")
                    .with_note(
                        "remove the object clone operation or assign it's result to $_ variable.",
                    );

                    vec![issue]
                }
                ObjectOperationExpression::MethodClosureCreation { .. } => {
                    let issue = Issue::warning(
                        AnalyzerIssueCode::DontDiscardObjectMethodClosureCreationOperation,
                        "object method closure creation discarded",
                    ).with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_note(
                        "object method closure creations are side-effect free, so you can safely remove them.",
                    );

                    vec![issue]
                }
                ObjectOperationExpression::PropertyFetch { .. } => {
                    let issue = Issue::warning(
                        AnalyzerIssueCode::DontDiscardObjectPropertyFetchOperation,
                        "object property fetch discarded",
                    ).with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_note(
                        "object property fetches are side-effect free, so you can safely remove them.",
                    );

                    vec![issue]
                }
                ObjectOperationExpression::NullsafePropertyFetch { .. } => {
                    let issue = Issue::warning(
                        AnalyzerIssueCode::DontDiscardObjectNullsafePropertyFetchOperation,
                        "object nullsafe property fetch discarded",
                    ).with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_note(
                        "object nullsafe property fetches are side-effect free, so you can safely remove them.",
                    );

                    vec![issue]
                }
                _ => vec![],
            },
            Expression::RangeOperation(_) => {
                let issue = Issue::warning(
                    AnalyzerIssueCode::DontDiscardRangeOperation,
                    "range operation discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )
                .with_note("range operations are side-effect free, so you can safely remove them.");

                vec![issue]
            }
            Expression::StringOperation(operation) => match operation {
                StringOperationExpression::Concat { .. } => {
                    let issue = Issue::warning(
                        AnalyzerIssueCode::DontDiscardStringConcatOperation,
                        "string concat discarded",
                    )
                    .with_source(
                        source,
                        expression.initial_position(),
                        expression.final_position(),
                    )
                    .with_note(
                        "string concats are side-effect free, so you can safely remove them.",
                    );

                    vec![issue]
                }
            },
            Expression::TypeOperation(_) => {
                let issue = Issue::warning(
                    AnalyzerIssueCode::DontDiscardTypeOperation,
                    "type operation discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                )
                .with_note("type operations are side-effect free, so you can safely remove them.");

                vec![issue]
            }
            Expression::AnonymousFunction(_) | Expression::ArrowFunction(_) => {
                let issue = Issue::warning(
                    AnalyzerIssueCode::DontDiscardAnonymousFunction,
                    "anonymous function discarded",
                )
                .with_source(
                    source,
                    expression.initial_position(),
                    expression.final_position(),
                );

                vec![issue]
            }
            _ => vec![],
        }
    }
}

impl Visitor for DiscardOperation {
    fn visit(&mut self, source: &str, node: &dyn Node, _ancestry: &[&dyn Node]) -> Vec<Issue> {
        let mut issues = vec![];

        if let Some(block) = downcast::<BlockStatement>(node) {
            for statement in &block.statements {
                if let Statement::Expression(expression) = statement {
                    for issue in Self::analyze_expression(source, &expression.expression) {
                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }
}
