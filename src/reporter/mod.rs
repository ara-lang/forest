use ara_reporting::builder::CharSet;
use ara_reporting::builder::ColorChoice;
use ara_reporting::builder::DisplayStyle;
use ara_reporting::builder::ReportBuilder;
use ara_reporting::Report;
use ara_source::SourceMap;

use crate::config::Configuration;
use crate::error::Error;

pub fn report(config: &Configuration, source_map: &SourceMap, report: Report) -> Result<(), Error> {
    if report.issues.is_empty() {
        return Ok(());
    }

    ReportBuilder::new(source_map, report)
        .with_colors({
            config
                .reporting
                .color
                .as_ref()
                .map(|s| match s.as_str() {
                    "auto" => ColorChoice::Auto,
                    "always" => ColorChoice::Always,
                    "never" => ColorChoice::Never,
                    _ => ColorChoice::Auto,
                })
                .unwrap_or(ColorChoice::Auto)
        })
        .with_charset({
            if let Some(true) = config.reporting.ascii {
                CharSet::Ascii
            } else {
                CharSet::Unicode
            }
        })
        .with_style({
            config
                .reporting
                .style
                .as_ref()
                .map(|s| match s.as_str() {
                    "default" => DisplayStyle::Default,
                    "comfortable" => DisplayStyle::Comfortable,
                    "compact" => DisplayStyle::Compact,
                    _ => DisplayStyle::Default,
                })
                .unwrap_or(DisplayStyle::Default)
        })
        .eprint()?;

    std::process::exit(1);
}
