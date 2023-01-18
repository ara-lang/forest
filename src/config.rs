use std::{collections::HashMap, path::PathBuf};

use config::{Config, Environment, File, ValueKind};
use serde_derive::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub root: PathBuf,
    pub project: ProjectConfiguration,
    pub definitions: Vec<DefinitionConfiguration>,
    pub reporting: ReportingConfiguration,
    pub analyzer: AnalyzerConfiguration,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfiguration {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct DefinitionConfiguration {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ReportingConfiguration {
    pub color: Option<String>,
    pub ascii: Option<bool>,
    pub style: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzerConfiguration {
    pub ignore: Vec<String>,
}

pub fn load(
    root: Option<&PathBuf>,
    path: Option<&PathBuf>,
    color: Option<&String>,
    ascii: Option<&bool>,
    style: Option<&String>,
    ignore: Vec<String>,
) -> Result<Configuration, Error> {
    let current = std::env::current_dir()?;

    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| current.to_path_buf());

    let root = if root.is_relative() {
        let mut absolute = current;
        absolute.push(root);

        absolute
    } else {
        root
    };

    let path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| ".ara.toml".into());

    let path = if !path.is_absolute() {
        let mut absolute = root.clone();
        absolute.push(path);

        absolute
    } else {
        path
    };

    let mut configuration: Configuration = Config::builder()
        .add_source(File::from(path).required(true))
        .add_source(Environment::with_prefix("ARA"))
        .set_override("root", root.to_string_lossy().to_string())?
        .set_default(
            "reporting",
            HashMap::from([
                ("color".to_string(), "auto"),
                ("ascii".to_string(), "false"),
                ("style".to_string(), "default"),
            ]),
        )?
        .set_default("definitions", ValueKind::Array(vec![]))?
        .set_default(
            "analyzer",
            HashMap::from([("ignore".to_string(), ValueKind::Array(vec![]))]),
        )?
        .build()?
        .try_deserialize()?;

    if let Some(color) = color {
        configuration.reporting.color = Some(color.clone());
    }

    if let Some(ascii) = ascii {
        configuration.reporting.ascii = Some(*ascii);
    }

    if let Some(style) = style {
        configuration.reporting.style = Some(style.clone());
    }

    for code in ignore {
        if !configuration.analyzer.ignore.contains(&code) {
            configuration.analyzer.ignore.push(code);
        }
    }

    Ok(configuration)
}
