mod addon_metadata;
mod binary;
mod error;
mod gma_builder;
mod gma_reader;
mod result;

pub use error::GMAError;
pub use gma_builder::{BuilderError, GMABuilder};
pub use gma_reader::{FileEntry, GMAFile};
pub use result::Result;

use gma_reader::GMAFileReader;

use std::io::BufReader;
use std::{
    io::{BufRead, Cursor, Seek},
    path::Path,
};

const IDENT: [u8; 4] = ['G' as u8, 'M' as u8, 'A' as u8, 'D' as u8];
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
pub enum Tag {
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

pub fn open<P>(path: P) -> Result<GMAFile<BufReader<std::fs::File>>>
where
    P: AsRef<Path>,
{
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    load(reader)
}

pub fn load<ReaderType>(r: ReaderType) -> Result<GMAFile<ReaderType>>
where
    ReaderType: BufRead + Seek,
{
    GMAFileReader::new(r).read_gma()
}

pub fn load_from_memory(data: &[u8]) -> Result<GMAFile<Cursor<&[u8]>>> {
    GMAFileReader::new(Cursor::new(data)).read_gma()
}
