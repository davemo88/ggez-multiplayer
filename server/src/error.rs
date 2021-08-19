use std::fmt;

#[derive(Debug)]
pub enum Error {
    NoPlayer(String),
    NoGame(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoGame(s) => write!(f, "NoGame({})", s),
            Error::NoPlayer(s) => write!(f, "NoPlayer({})", s)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::NoGame(_) => None,
            Error::NoPlayer(_) => None
        }
    }
}
