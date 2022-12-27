use std::fmt::Display;

#[derive(Debug)]
pub enum ConsoleError {
    ParseError,
    MismatchQuotes,

    Unknown,

    ClapError(String),
}

impl Display for ConsoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError => writeln!(f, "error while parsing line"),
            Self::MismatchQuotes => writeln!(f, "error due to mismatch quotes"),
            Self::ClapError(err) => writeln!(f, "{}", err),
            Self::Unknown => writeln!(f, "unknown command"),
            _ => writeln!(f, "unknown error"),
        }
    }
}
