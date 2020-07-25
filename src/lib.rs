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
use std::convert::TryFrom;

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

impl TryFrom<&str> for AddonType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value_lower = value.to_lowercase();
        match value_lower.as_str() {
            "gamemode" => Ok(AddonType::Gamemode),
            "map" => Ok(AddonType::Map),
            "weapon" => Ok(AddonType::Weapon),
            "vehicle" => Ok(AddonType::Vehicle),
            "npc" => Ok(AddonType::NPC),
            "entity" => Ok(AddonType::Entity),
            "tool" => Ok(AddonType::Tool),
            "effects" => Ok(AddonType::Effects),
            "model" => Ok(AddonType::Model),
            "servercontent" => Ok(AddonType::ServerContent),
            _ => Err(Self::Error::InvalidAddonType(value_lower)),
        }
    }
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

impl TryFrom<&str> for AddonTag {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value_lower = value.to_lowercase();
        match value_lower.as_str() {
            "fun" => Ok(AddonTag::Fun),
            "roleplay" => Ok(AddonTag::Roleplay),
            "scenic" => Ok(AddonTag::Scenic),
            "movie" => Ok(AddonTag::Movie),
            "realism" => Ok(AddonTag::Realism),
            "cartoon" => Ok(AddonTag::Cartoon),
            "water" => Ok(AddonTag::Water),
            "comic" => Ok(AddonTag::Comic),
            "build" => Ok(AddonTag::Build),
            _ => Err(Self::Error::InvalidAddonTag(value_lower)),
        }
    }
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
    GMAFileReader::new(r)?.read_gma()
}

/// Loads a gma file from memory
pub fn load_from_memory(data: &[u8]) -> Result<GMAFile<Cursor<&[u8]>>> {
    load(Cursor::new(data))
}
