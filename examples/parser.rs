use std::env;

use ara_forest::config::Config;
use ara_forest::logger::LogLevel;
use ara_forest::logger::Logger;
use ara_forest::Parser;
use ara_reporting::Report;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> Result<(), Box<Report>> {
    let root = format!("{MANIFEST_DIR}/examples/project");

    let config = Config::new(root)
        .with_source("src")
        .with_logger(Logger::new().with_level(LogLevel::Error))
        .with_cache_directory(".cache");

    let forest = Parser::new(&config).parse()?;

    assert_eq!(forest.source.sources.len(), 3000);
    assert_eq!(forest.tree.trees.len(), 3000);

    Ok(())
}
