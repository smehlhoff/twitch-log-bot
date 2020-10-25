use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    Postgres(postgres::Error),
    Regex(regex::Error),
    Irc(irc::error::IrcError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Io(ref err) => write!(f, "{}", err),
            Self::Json(ref err) => write!(f, "{}", err),
            Self::Postgres(ref err) => write!(f, "{}", err),
            Self::Regex(ref err) => write!(f, "{}", err),
            Self::Irc(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<postgres::Error> for Error {
    fn from(err: postgres::Error) -> Self {
        Self::Postgres(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Self::Regex(err)
    }
}

impl From<irc::error::IrcError> for Error {
    fn from(err: irc::error::IrcError) -> Self {
        Self::Irc(err)
    }
}
