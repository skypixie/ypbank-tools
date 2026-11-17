use crate::error::Error;

/// Format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Csv,
    Text,
    Bin,
}

/// Convert string (for example, command line argument) to input format enum
pub fn string_to_format(s: &str) -> Result<Format, Error> {
    match s {
        "csv" => Ok(Format::Csv),
        "text" => Ok(Format::Text),
        "bin" => Ok(Format::Bin),
        &_ => Err(Error::UnknownFormat(s.to_string())),
    }
}
