use ara_reporting::issue::Issue;
use ara_reporting::Report;

#[derive(Debug)]
pub enum Error {
    EncodeError(String),
    DecodeError(String),
    InvalidPath(String),
    IoError(std::io::Error),
    ParseError(Box<Report>),
    LogError(log::SetLoggerError),
}

impl From<walkdir::Error> for Error {
    fn from(error: walkdir::Error) -> Self {
        Error::IoError(error.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<Error> for Report {
    fn from(error: Error) -> Self {
        Report::new().with_issue(Issue::from_string(error.to_string()))
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(error: bincode::error::EncodeError) -> Self {
        Error::EncodeError(error.to_string())
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(error: bincode::error::DecodeError) -> Self {
        Error::DecodeError(error.to_string())
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(error: log::SetLoggerError) -> Self {
        Error::LogError(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(error) => write!(f, "io error: {error}"),
            Error::InvalidPath(message) => write!(f, "invalid source: {message}"),
            Error::EncodeError(message) => write!(f, "encode error: {message}"),
            Error::DecodeError(message) => write!(f, "decode error: {message}"),
            Error::ParseError(report) => write!(f, "parse error: {report}"),
            Error::LogError(error) => write!(f, "log error: {error}"),
        }
    }
}
