use crate::binary;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidString, // this is likely due to trying to write a string containing a null byte
    UTF8Error(std::string::FromUtf8Error),
    InvalidIdent,
    InvalidVersion(u8),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::UTF8Error(e)
    }
}

impl From<binary::Error> for Error {
    fn from(e: binary::Error) -> Self {
        match e {
            binary::Error::IOError(e) => Self::IOError(e),
            binary::Error::InvalidUTF8(e) => Self::UTF8Error(e),
            binary::Error::InvalidCString => Self::InvalidString,
        }
    }
}
