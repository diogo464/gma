use crate::binary;
use lzma_rs;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidString, // this is likely due to trying to write a string containing a null byte
    UTF8Error(std::string::FromUtf8Error),
    InvalidIdent,
    InvalidVersion(u8),
    CompressionError(lzma_rs::error::Error),
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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(e) => e.fmt(f),
            Self::InvalidString => write!(f, "An invalid string was found, strings cant have the null byte and should only contain ascii characters"),
            Self::UTF8Error(e) => e.fmt(f),
            Self::InvalidIdent => write!(f, "The gma file did not containt a valid ident, 'GMAD' was expect at the start of the file"),
            Self::InvalidVersion(v) => write!(f, "An invalid version of gma file was found : '{}', this might be cause by a corrupt file", v),
            Self::CompressionError(e) => write!(f, "Error while compressing/decompressing. {:?}", e),
        }
    }
}

impl std::error::Error for Error {}
