use simplelog::*;
use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;

use crate::error::Error;

#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug)]
pub struct Logger {
    level: Option<LogLevel>,
    file: Option<PathBuf>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            level: None,
            file: None,
        }
    }

    #[must_use]
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = Some(level);

        self
    }

    #[must_use]
    pub fn with_file(mut self, path: PathBuf) -> Self {
        self.file = Some(path);

        self
    }

    pub fn init(&self) -> Result<(), Error> {
        let level = self.level.unwrap_or(LogLevel::Off);

        let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
            level.into(),
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )];

        if self.file.is_some() {
            loggers.push(WriteLogger::new(
                level.into(),
                Config::default(),
                File::create(self.file.as_ref().unwrap())?,
            ));
        }

        CombinedLogger::init(loggers)?;

        Ok(())
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}
