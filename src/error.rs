use crate::binary;

#[derive(Debug)]
pub enum GMAError {
    IOError(std::io::Error),
    UTF8Error(std::string::FromUtf8Error),
    InvalidIdent,
    InvalidVersion(u8),
}

impl From<std::io::Error> for GMAError {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<std::string::FromUtf8Error> for GMAError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::UTF8Error(e)
    }
}

impl From<binary::Error> for GMAError {
    fn from(e: binary::Error) -> Self {
        match e {
            binary::Error::IOError(e) => Self::IOError(e),
            binary::Error::InvalidUTF8(e) => Self::UTF8Error(e),
        }
    }
}

//TODO: use the same error type in the entire crate
