use clap::arg;
use clap::Command;
use std::path::PathBuf;

use ara_parser::parser;

use crate::analyzer::Analyzer;
use crate::error::Result;

pub mod analyzer;
pub mod config;
pub mod error;
pub mod reporter;
pub mod source;

fn main() -> Result<()> {
    let command = Command::new("ara-analyzer")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Saif Eddin Gmati <azjezz@protonmail.com>")
        .about("Linter, Type Checker, and Static Analyzer for Ara Programming Language 🐦")
        .arg_required_else_help(true)
        .args([
            arg!(--project <PATH>)
                .long("project")
                .help("The path to the project directory.")
                .value_parser(clap::value_parser!(PathBuf))
                .num_args(0..=1),
            arg!(--config <PATH>)
                .long("config")
                .help("The path to the configuration file.")
                .value_parser(clap::value_parser!(PathBuf))
                .num_args(0..=1),
            arg!(--color <WHEN>)
                .long("color")
                .help("Coloring of output.")
                .value_parser(["always", "auto", "never"])
                .num_args(1),
            arg!(--ascii)
                .long("ascii")
                .help("Disables unicode characters.")
                .value_parser(clap::value_parser!(bool)),
            arg!(--style <STYLE>)
                .long("style")
                .help("The reporting output style.")
                .value_parser(["default", "compact", "comfortable"])
                .num_args(1),
            arg!(--ignore <CODES>...)
                .help("Ignores the given issue codes.")
                .value_parser(clap::value_parser!(String))
                .value_delimiter(',')
                .num_args(0..=1),
        ]);

    let matches = command.get_matches();

    let config = config::load(
        matches.get_one::<PathBuf>("project"),
        matches.get_one::<PathBuf>("config"),
        matches.get_one::<String>("color"),
        matches.get_one::<bool>("ascii"),
        matches.get_one::<String>("style"),
        matches
            .get_many::<String>("ignore")
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect(),
    )?;

    let source_map = source::load(&config)?;
    let tree_map = match parser::parse_map(&source_map) {
        Ok(tree_map) => tree_map,
        Err(report) => {
            return reporter::report(&config, &source_map, *report);
        }
    };

    let analyzer = Analyzer::new(&config);
    let analysis_report = analyzer.analyze(&source_map, &tree_map)?;

    if let Some(report) = analysis_report.report {
        reporter::report(&config, &source_map, report)
    } else {
        Ok(())
    }
}
