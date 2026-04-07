use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidVersion(String),
    UnsupportedPlatform(String),
    VersionNotFound(String),
    NotInstalled(String),
    Msg(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(formatter, "{err}"),
            Error::InvalidVersion(version) => write!(
                formatter,
                "invalid version '{version}'. expected e.g. 1.19.1 or 1.20.0-rc.1"
            ),
            Error::UnsupportedPlatform(platform) => {
                write!(formatter, "unsupported platform: {platform}")
            }
            Error::VersionNotFound(version) => write!(
                formatter,
                "IOTA version '{version}' not found. check https://github.com/iotaledger/iota/releases"
            ),
            Error::NotInstalled(version) => write!(
                formatter,
                "{version} is not installed. try: iotaup install {version}"
            ),
            Error::Msg(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
