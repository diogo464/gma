//! Crate for reading and writing gma files, the file format of garrys mod's addons.
//! This crate currently does not support opening compressed archives.

mod addon_metadata;
mod binary;
mod error;
mod gma_builder;
mod gma_reader;
mod result;

pub use error::Error;
pub use gma_builder::GMABuilder;
pub use gma_reader::{FileEntry, GMAFile};
pub use result::Result;

use gma_reader::GMAFileReader;

use std::io::BufReader;
use std::{
    io::{BufRead, Cursor, Seek},
    path::Path,
};

const IDENT: [u8; 4] = [b'G', b'M', b'A', b'D'];
const VALID_VERSIONS: [u8; 3] = [1, 2, 3];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddonType {
    Gamemode,
    Map,
    Weapon,
    Vehicle,
    NPC,
    Entity,
    Tool,
    Effects,
    Model,
    ServerContent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddonTag {
    Fun,
    Roleplay,
    Scenic,
    Movie,
    Realism,
    Cartoon,
    Water,
    Comic,
    Build,
}

/// Opens a file from disk with the given path and tries to read it as a gma archive
pub fn open<P>(path: P) -> Result<GMAFile<BufReader<std::fs::File>>>
where
    P: AsRef<Path>,
{
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    load(reader)
}

/// Loads a gma file from a reader
pub fn load<ReaderType>(r: ReaderType) -> Result<GMAFile<ReaderType>>
where
    ReaderType: BufRead + Seek,
{
    GMAFileReader::new(r).read_gma()
}

/// Loads a gma file from memory
pub fn load_from_memory(data: &[u8]) -> Result<GMAFile<Cursor<&[u8]>>> {
    GMAFileReader::new(Cursor::new(data)).read_gma()
}
