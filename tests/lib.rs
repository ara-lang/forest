use std::env;

use ara_forest::config::Config;
use ara_forest::logger::{LogLevel, Logger};
use ara_forest::Parser;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn test_parsing_project_a_with_log() {
    let root = format!("{MANIFEST_DIR}/tests/examples/project-a");

    let config = Config::new(root)
        .with_source("src")
        .with_definitions(vec![
            format!("vendor/std-bar/definitions"),
            format!("vendor/std-foo/definitions"),
        ])
        .with_cache_directory(".cache")
        .with_logger(Logger::new().with_level(LogLevel::Debug));

    let forest = Parser::new(&config).parse().unwrap();

    assert_eq!(forest.source.sources.len(), 6);
    assert_eq!(forest.tree.trees.len(), 6);
}

#[test]
fn test_parsing_empty_project() {
    let root = format!("{MANIFEST_DIR}/tests/examples/project-empty");

    let config = Config::new(root).with_source("src");

    Parser::new(&config)
        .parse()
        .expect_err("Expected an error, but got a Forest object");
}

#[test]
fn test_parsing_project_with_parse_error() {
    let root = format!("{MANIFEST_DIR}/tests/examples/project-b");

    let config = Config::new(root).with_source("src");

    let report = Parser::new(&config)
        .parse()
        .expect_err("Expected an error Report, but got a Forest object");

    assert!(report
        .issues
        .first()
        .unwrap()
        .message
        .contains("unexpected token `||`"));
}
