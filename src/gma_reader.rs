use crate::{
    addon_metadata::AddonMetadata, binary::BinaryReader, AddonType, GMAError, Result, Tag, IDENT,
    VALID_VERSIONS,
};
use std::{
    cell::RefCell,
    io::{BufRead, Read, Seek, SeekFrom},
};

#[derive(Debug)]
pub struct FileEntry {
    filename: String,
    filesize: u64,
    crc: u32,
    offset: u64,
}

impl FileEntry {
    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn size(&self) -> u64 {
        self.filesize
    }
    pub fn crc(&self) -> u32 {
        self.crc
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

#[derive(Debug)]
pub struct GMAFile<ReaderType>
where
    ReaderType: BufRead + Seek,
{
    version: u8,
    steamid: u64,
    timestamp: u64,
    name: String,
    description: String,
    addon_type: Option<AddonType>,
    addon_tags: Vec<Tag>,
    author: String,
    entries: Vec<FileEntry>,
    file_data_start: u64,
    reader: RefCell<Option<ReaderType>>,
}

impl<ReaderType> GMAFile<ReaderType>
where
    ReaderType: BufRead + Seek,
{
    pub fn version(&self) -> u8 {
        self.version
    }
    pub fn appid(&self) -> u32 {
        4000 // this is the gmod appid
    }
    pub fn author_steamid(&self) -> u64 {
        self.steamid
    }
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn addon_type(&self) -> Option<AddonType> {
        self.addon_type
    }
    pub fn addon_tags(&self) -> &[Tag] {
        &self.addon_tags
    }
    pub fn contains_tag(&self, tag: Tag) -> bool {
        self.addon_tags.contains(&tag)
    }
    pub fn author(&self) -> &str {
        &self.author
    }
    pub fn entries(&self) -> impl Iterator<Item = &FileEntry> {
        self.entries.iter()
    }
    pub fn read_entry<F, R>(&self, entry: &FileEntry, func: F) -> Result<R>
    where
        F: FnOnce(&FileEntry, &mut dyn Read) -> R,
    {
        //this doesnt look good
        let mut reader = self.reader.replace(None).unwrap();
        reader.seek(std::io::SeekFrom::Start(
            self.file_data_start + entry.offset,
        ))?;
        let mut entry_reader = (&mut reader).take(entry.filesize);
        let result = func(entry, &mut entry_reader);
        self.reader.replace(Some(reader));
        Ok(result)
    }
}

pub struct GMAFileReader<ReaderType>
where
    ReaderType: BufRead + Seek,
{
    reader: ReaderType,
}

impl<ReaderType> GMAFileReader<ReaderType>
where
    ReaderType: BufRead + Seek,
{
    pub fn new(reader: ReaderType) -> Self {
        Self { reader }
    }

    pub fn read_gma(mut self) -> Result<GMAFile<ReaderType>> {
        self.read_ident()?;
        let version = self.read_version()?;
        let steamid = self.read_steamid()?;
        let timestamp = self.read_timestamp()?;

        if version > 1 {
            //unused right now
            self.read_required_content()?;
        }

        let name = self.read_name()?;
        let metadata_str = self.read_desc()?;
        let author = self.read_author()?;

        let _addon_version = self.read_addon_version()?;
        let entries = self.read_file_entries()?;
        let file_data_start = self.reader.seek(SeekFrom::Current(0))?;
        let (desc, ty, tags) = if let Some(metadata) = AddonMetadata::from_json(&metadata_str) {
            let ty = metadata.get_type();
            let mut tags = Vec::new();
            let (t1, t2) = metadata.get_tags();
            let desc = metadata.get_description().to_owned();
            if let Some(t1) = t1 {
                tags.push(t1);
            }
            if let Some(t2) = t2 {
                tags.push(t2);
            }

            (desc, ty, tags)
        } else {
            (metadata_str, None, Vec::new())
        };

        Ok(GMAFile {
            version,
            steamid,
            timestamp,
            name,
            description: desc,
            addon_type: ty,
            addon_tags: tags,
            author,
            entries,
            file_data_start: file_data_start as u64,
            reader: RefCell::new(Some(self.reader)),
        })
    }

    fn read_ident(&mut self) -> Result<()> {
        let mut ident: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut ident)?;
        if ident != IDENT {
            Err(GMAError::InvalidIdent)
        } else {
            Ok(())
        }
    }

    fn read_version(&mut self) -> Result<u8> {
        let version = self.reader.read_u8()?.1;
        if !VALID_VERSIONS.contains(&version) {
            Err(GMAError::InvalidVersion(version))
        } else {
            Ok(version)
        }
    }

    fn read_steamid(&mut self) -> Result<u64> {
        Ok(self.reader.read_u64()?.1)
    }

    fn read_timestamp(&mut self) -> Result<u64> {
        Ok(self.reader.read_u64()?.1)
    }

    fn read_required_content(&mut self) -> Result<Vec<String>> {
        let mut v = Vec::new();
        while {
            let string = self.reader.read_c_string()?.1;
            v.push(string);
            v.last().unwrap().len() != 0
        } {}
        Ok(v)
    }

    fn read_name(&mut self) -> Result<String> {
        Ok(self.reader.read_c_string()?.1)
    }

    fn read_desc(&mut self) -> Result<String> {
        Ok(self.reader.read_c_string()?.1)
    }

    fn read_author(&mut self) -> Result<String> {
        Ok(self.reader.read_c_string()?.1)
    }

    fn read_addon_version(&mut self) -> Result<u32> {
        Ok(self.reader.read_u32()?.1)
    }

    fn read_file_entries(&mut self) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        let mut current_offset: u64 = 0;
        while self.reader.read_u32()?.1 != 0 {
            let filename = self.reader.read_c_string()?.1;
            let filesize = self.reader.read_u64()?.1;
            let crc = self.reader.read_u32()?.1;
            let offset = current_offset;
            current_offset += filesize;
            entries.push(FileEntry {
                filename,
                filesize,
                crc,
                offset,
            })
        }
        Ok(entries)
    }
}
