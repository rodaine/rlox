use std::result;
use std::error;
use std::fmt;
use std::io;

pub type Result<T> = result::Result<T, Box<error::Error>>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Usage,
    IO(io::Error),
    Lexical(u64, String, String),
}

impl Error {
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
