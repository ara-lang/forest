use ara_reporting::error::Error as ReportingError;
use ara_source::error::Error as SourceError;
use config::ConfigError;
use std::io::Error as IoError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(IoError),
    Reporting(ReportingError),
    Source(SourceError),
    Config(ConfigError),
    Parsing,
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::IO(error)
    }
}

impl From<ReportingError> for Error {
    fn from(error: ReportingError) -> Self {
        Error::Reporting(error)
    }
}

impl From<SourceError> for Error {
    fn from(error: SourceError) -> Self {
        Error::Source(error)
    }
}

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Self {
        Error::Config(error)
    }
}
