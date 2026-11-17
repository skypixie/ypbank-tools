use std::fmt;

/// Lib errors for reading, writing and data parsing
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    InvalidDateFormat(String),
    BadNumber(String),
    UnknownKind(String),
    CorruptedBinaryFile(String),
    UnknownFormat(String),
    InvalidLine(String),
    InvalidHeader(String),
    StringTooLong(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(error) => write!(f, "IO Error: {error}"),
            Error::InvalidDateFormat(inv_date) => write!(f, "Invalid date format: {inv_date}"),
            Error::BadNumber(number_str) => write!(f, "Bad number: {number_str}"),
            Error::UnknownKind(kind_str) => write!(f, "Unknown kind: {kind_str}"),
            Error::CorruptedBinaryFile(reason) => write!(f, "Corrupted binary file: {reason}"),
            Error::UnknownFormat(format) => write!(f, "Unknown format: {format}"),
            Error::InvalidLine(line) => write!(f, "Cannot parse line: {line}"),
            Error::InvalidHeader(header) => write!(f, "Invalid header: {header}"),
            Error::StringTooLong(s) => write!(f, "String too long: {s}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}
