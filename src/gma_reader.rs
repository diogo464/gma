use crate::{
    addon_metadata::AddonMetadata, binary::BinaryReader, AddonTag, AddonType, Error, Result, IDENT,
    VALID_VERSIONS,
};
use lzma_rs;
use std::{
    cell::RefCell,
    io::{BufRead, Cursor, Read, Seek, SeekFrom},
};

/// GMA File Entry
#[derive(Debug)]
pub struct FileEntry {
    filename: String,
    filesize: u64,
    crc: u32,
    offset: u64,
}

impl FileEntry {
    /// The full filename of this entry. Ex : lua/autorun/cl_myscript.lua
    pub fn filename(&self) -> &str {
        &self.filename
    }
    /// The file size
    pub fn size(&self) -> u64 {
        self.filesize
    }
    /// The crc32 of this entry's contents
    pub fn crc(&self) -> u32 {
        self.crc
    }
    /// The offset in the gma file, starting from the first file
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

#[derive(Debug)]
enum StreamType<R>
where
    R: BufRead + Seek,
{
    Compressed((R, Cursor<Vec<u8>>)),
    Uncompressed(R),
}
impl<R> Read for StreamType<R>
where
    R: Seek + BufRead,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Compressed((_, r)) => r.read(buf),
            Self::Uncompressed(r) => r.read(buf),
        }
    }
}
impl<R> BufRead for StreamType<R>
where
    R: BufRead + Seek,
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            Self::Compressed((_, r)) => r.fill_buf(),
            Self::Uncompressed(r) => r.fill_buf(),
        }
    }
    fn consume(&mut self, amt: usize) {
        match self {
            Self::Compressed((_, r)) => r.consume(amt),
            Self::Uncompressed(r) => r.consume(amt),
        }
    }
}
impl<R> Seek for StreamType<R>
where
    R: Seek + BufRead,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::Compressed((_, r)) => r.seek(pos),
            Self::Uncompressed(r) => r.seek(pos),
        }
    }
}

/// GMA File
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
    addon_tags: Vec<AddonTag>,
    author: String,
    entries: Vec<FileEntry>,
    file_data_start: u64,
    reader: RefCell<Option<StreamType<ReaderType>>>,
}

impl<ReaderType> GMAFile<ReaderType>
where
    ReaderType: BufRead + Seek,
{
    /// Get the gma archive versiom
    pub fn version(&self) -> u8 {
        self.version
    }
    /// The appid. This is always '4000', the appid of garry's mod
    pub fn appid(&self) -> u32 {
        4000 // this is the gmod appid
    }
    /// The author's steamid. This is currently unused by the game and is usually hardcoded to 0
    pub fn author_steamid(&self) -> u64 {
        self.steamid
    }
    /// The seconds since UNIX epoch from when the file was created
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    /// The name of the addon
    pub fn name(&self) -> &str {
        &self.name
    }
    /// The description of the addon
    pub fn description(&self) -> &str {
        &self.description
    }
    /// The type of the addon
    pub fn addon_type(&self) -> Option<AddonType> {
        self.addon_type
    }
    /// The tags of the item. This should be at most 2 but this implementation supports reading more
    pub fn addon_tags(&self) -> &[AddonTag] {
        &self.addon_tags
    }
    /// Helper function to check if this addon contains a certain tag
    pub fn contains_tag(&self, tag: AddonTag) -> bool {
        self.addon_tags.contains(&tag)
    }
    /// The name of the addon's author
    pub fn author(&self) -> &str {
        &self.author
    }
    /// Returns true if the input file was compressed, false otherwise
    pub fn compressed(&self) -> bool {
        match self
            .reader
            .borrow()
            .as_ref()
            .expect("The reader should not be None, this is a bug")
        {
            StreamType::Compressed(_) => true,
            StreamType::Uncompressed(_) => false,
        }
    }
    /// An iterator of the file entries of this archive
    pub fn entries(&self) -> impl Iterator<Item = &FileEntry> {
        self.entries.iter()
    }
    /// Function to read the contents of a given entry.
    ///
    /// The callback function takes as parameter a reference to the entry and a mutable
    /// reference to a type that implements Read.
    /// ```
    /// use std::io::Read;
    /// # let dummy_buffer = &include_bytes!("../tests/addon.gma")[..];
    /// let archive = gma::load_from_memory(&dummy_buffer).unwrap();
    /// for entry in archive.entries() {
    ///     let contents = archive.read_entry(entry, |entry_ref, reader|{
    ///         let mut c = String::new();
    ///         reader.read_to_string(&mut c).unwrap();
    ///         c
    ///     }).unwrap();
    ///     // do something with contents
    /// }
    pub fn read_entry<F, R>(&self, entry: &FileEntry, func: F) -> Result<R>
    where
        F: FnOnce(&FileEntry, &mut dyn Read) -> R,
    {
        //this doesnt look good
        let mut stream = self.reader.replace(None).unwrap();
        //TODO: if there is a problem with seek we lose the reader
        stream.seek(std::io::SeekFrom::Start(
            self.file_data_start + entry.offset,
        ))?;
        let mut entry_reader = (&mut stream).take(entry.filesize);
        let result = func(entry, &mut entry_reader);
        self.reader.replace(Some(stream));
        Ok(result)
    }
}

pub struct GMAFileReader<ReaderType>
where
    ReaderType: BufRead + Seek,
{
    reader: StreamType<ReaderType>,
}

impl<ReaderType> GMAFileReader<ReaderType>
where
    ReaderType: BufRead + Seek,
{
    pub fn new(reader: ReaderType) -> Result<Self> {
        Ok(Self {
            reader: get_reader_stream(reader)?,
        })
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
            Err(Error::InvalidIdent)
        } else {
            Ok(())
        }
    }

    fn read_version(&mut self) -> Result<u8> {
        let version = self.reader.read_u8()?.1;
        if !VALID_VERSIONS.contains(&version) {
            Err(Error::InvalidVersion(version))
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
            !v.last().unwrap().is_empty()
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

// Returns a decompression stream if the provided stream is lzma compressed,
// otherwise returns the provided stream
fn get_reader_stream<ReaderType>(mut reader: ReaderType) -> Result<StreamType<ReaderType>>
where
    ReaderType: BufRead + Seek,
{
    let mut probe_buffer: [u8; 4] = [0; 4];
    let stream_start_pos = reader.seek(SeekFrom::Current(0))?;
    reader.read_exact(&mut probe_buffer)?;
    reader.seek(SeekFrom::Start(stream_start_pos))?;
    match probe_buffer {
        IDENT => Ok(StreamType::Uncompressed(reader)),
        //Error decompressing, we assume this is not a lzma file
        _ => {
            let file_buffer = Vec::new();
            let mut buffer_cursor = Cursor::new(file_buffer);
            lzma_rs::lzma_decompress(&mut reader, &mut buffer_cursor).unwrap();
            buffer_cursor.seek(SeekFrom::Start(0))?;
            Ok(StreamType::Compressed((reader, buffer_cursor)))
        }
    }
}
