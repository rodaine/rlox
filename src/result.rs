//! A module describing Lox-specific Result and Error types

use std::result;
use std::error;
use std::fmt;
use std::io;

/// A Lox-Specific Result Type
pub type Result<T> = result::Result<T, Box<error::Error>>;

/// A Lox-Specific Error
#[derive(Debug)]
pub enum Error {
    /// Returned if the CLI command is used incorrectly
    Usage,
    /// Returned if there is an error reading from a file or stdin
    IO(io::Error),
    /// Returned if the scanner encounters an error
    Lexical(u64, String, String),
}

impl Error {
    /// Returns a boxed version of this error, useful for creating a valid Result
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rlox;
    /// # use rlox::*;
    /// # use rlox::Error::*;
    /// # fn main() {
    /// let res : Result<()> = Err(Usage.boxed());
    /// # }
    /// ```
    pub fn boxed(self) -> Box<Error> {
        Box::new(self)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Usage => write!(f, "Usage: rlox [script]"),
            Error::IO(ref e) => e.fmt(f),
            Error::Lexical(ref line, ref whence, ref msg) =>
                write!(f, "[line {}] Error {}: {:?}", line, whence, msg),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Usage => "Usage: rlox [script]",
            Error::IO(ref e) => e.description(),
            Error::Lexical(_, _, _) => "lexical error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IO(ref e) => e.cause(),
            _ => None,
        }
    }
}
