use crate::binary::BinaryWriter;
use crate::{addon_metadata::AddonMetadata, result::Result, AddonTag, AddonType, Error, IDENT};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::{
    fs::File,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

enum BuilderFileReader<'a> {
    FSFile(File),
    Bytes(&'a [u8]),
    Reader(&'a mut dyn Read),
}

struct BuilderFile<'a> {
    filename: String,
    reader: BuilderFileReader<'a>,
}

struct FilePatchInfo {
    filesize: u64,
    crc: u32,
}

pub struct GMABuilder<'a> {
    version: u8,
    steamid: u64,
    timestamp: u64,
    name: &'a str,
    description: &'a str,
    author: &'a str,
    files: Vec<BuilderFile<'a>>,
    addon_type: AddonType,
    addon_tags: [Option<AddonTag>; 2],
}

impl<'a> GMABuilder<'a> {
    /// Creates a new gma builder
    pub fn new() -> Self {
        Self {
            version: 3,
            steamid: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::new(0, 0))
                .as_secs() as u64,
            name: "GMABuilder addon name",
            description: "A description",
            author: "GMABuilder",
            files: Vec::new(),
            addon_type: AddonType::Tool,
            addon_tags: [None; 2],
        }
    }

    /// Sets the gma version of the archive. Default : 3
    pub fn version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    /// Sets the steamid of the author
    pub fn steamid(mut self, steamid: u64) -> Self {
        self.steamid = steamid;
        self
    }

    /// Sets the timestamp. Default : current time
    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the name of the addon
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    /// Sets the description of the addon
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    /// Sets the name of the author
    pub fn author(mut self, author: &'a str) -> Self {
        self.author = author;
        self
    }

    /// Sets the addon type
    pub fn addon_type(mut self, addon_type: AddonType) -> Self {
        self.addon_type = addon_type;
        self
    }

    /// Adds tag to the addon.
    /// Only 2 tags are allowed at any given time, adding more will replace the oldest one
    pub fn addon_tag(mut self, addon_tag: AddonTag) -> Self {
        let (avail1, avail2) = (self.addon_tags[0].is_none(), self.addon_tags[1].is_none());
        match (avail1, avail2) {
            (false, false) | (true, true) => {
                self.addon_tags[1] = self.addon_tags[0];
                self.addon_tags[0] = Some(addon_tag)
            }
            //2nd case on line bellow should never happen
            (true, false) | (false, true) => self.addon_tags[1] = Some(addon_tag),
        };
        self
    }

    /// Adds a file to the archive from the provided path
    pub fn file_from_path<S: Into<String>>(mut self, path: S) -> Self {
        let path = path.into();
        let file = File::open(&path).unwrap();
        self.files.push(BuilderFile {
            filename: path,
            reader: BuilderFileReader::FSFile(file),
        });
        self
    }

    /// Adds a file with the given filename and contents
    pub fn file_from_bytes<S: Into<String>>(mut self, filename: S, bytes: &'a [u8]) -> Self {
        self.files.push(BuilderFile {
            filename: filename.into(),
            reader: BuilderFileReader::Bytes(bytes),
        });
        self
    }

    /// Adds a file with the given filename and contents are read from `reader`
    pub fn file_from_reader<S: Into<String>>(
        mut self,
        filename: S,
        reader: &'a mut dyn Read,
    ) -> Self {
        self.files.push(BuilderFile {
            filename: filename.into(),
            reader: BuilderFileReader::Reader(reader),
        });
        self
    }

    /// Consumes the builder and writes the gma file contents to the given `writer`
    pub fn write_to<WriterType>(self, mut writer: WriterType) -> Result<()>
    where
        WriterType: Write + Seek,
    {
        Self::write_ident(&mut writer)?;
        //write version
        writer.write_u8(self.version)?;
        //write steamid
        writer.write_u64(self.steamid)?;
        //write timestamp
        writer.write_u64(self.timestamp)?;
        //write required contents
        //this is unused right now so just write an empty string
        writer.write_u8(0)?;
        //write addon name
        writer.write_c_string(self.name)?;
        //write metadata string
        let tags: Vec<AddonTag> = self
            .addon_tags
            .iter()
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect();
        let metadata = AddonMetadata::new(
            self.name.to_owned(),
            self.description.to_owned(),
            &self.addon_type,
            &tags,
        );
        let metadata_json = metadata.to_json();
        writer.write_c_string(&metadata_json)?;
        //write author name
        writer.write_c_string(self.author)?;
        //write addon_version
        //this is currently unused and should be set to 1
        writer.write_u32(1)?;

        //write file entries
        //absolute offsets inside the writer
        let mut patch_offsets = Vec::with_capacity(self.files.len());
        let mut patch_info = Vec::with_capacity(self.files.len());
        for (i, entry) in self.files.iter().enumerate() {
            let file_number = (i + 1) as u32;
            let (_, patch_offset) =
                Self::write_incomplete_file_entry(&mut writer, file_number, &entry)?;
            patch_offsets.push(patch_offset);
        }
        //we need to write a 0 to indicate the end of file entries
        writer.write_u32(0)?;
        for entry in self.files.into_iter() {
            let (_, patch) = Self::write_file_contents(&mut writer, entry)?;
            patch_info.push(patch)
        }
        assert_eq!(patch_info.len(), patch_offsets.len());
        for (offset, info) in patch_offsets.into_iter().zip(patch_info.into_iter()) {
            Self::apply_file_entry_patch(&mut writer, offset, info)?;
        }

        Ok(())
    }

    fn write_ident<WriterType: Write>(mut writer: WriterType) -> Result<usize> {
        Ok(writer.write(&IDENT)?)
    }

    //Returns the amount of bytes written and the offset to the filesize field so we can patch it later
    fn write_incomplete_file_entry<WriterType: Write + Seek>(
        mut writer: WriterType,
        file_number: u32,
        bfile: &BuilderFile,
    ) -> Result<(usize, u64)> {
        let mut bytes_written = 0;
        bytes_written += writer.write_u32(file_number)?;
        bytes_written += writer.write_c_string(&bfile.filename)?;
        //write filesize, crc32 and offset. We will patch this values later
        let offset_to_patch_start = writer.seek(SeekFrom::Current(0))?;
        bytes_written += writer.write_u64(0)?;
        bytes_written += writer.write_u32(0)?;
        Ok((bytes_written, offset_to_patch_start))
    }

    fn write_file_contents<WriterType: Write + Seek>(
        mut writer: WriterType,
        bfile: BuilderFile,
    ) -> Result<(usize, FilePatchInfo)> {
        let mut write_contents = |reader: &mut dyn Read| -> Result<(usize, FilePatchInfo)> {
            const BLOCK_SIZE: usize = 8096;
            let mut bytes_written: usize = 0;
            let mut buffer: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
            loop {
                let read_result = reader.read(&mut buffer);
                match read_result {
                    Ok(0) => {
                        //TODO: add crc32
                        return Ok((
                            bytes_written,
                            FilePatchInfo {
                                filesize: bytes_written as u64,
                                crc: 0,
                            },
                        ));
                    }
                    Ok(n) => {
                        writer.write_all(&buffer[0..n])?;
                        bytes_written += n;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(Error::IOError(e)),
                }
            }
        };
        match bfile.reader {
            BuilderFileReader::FSFile(file) => {
                let mut reader = BufReader::new(file);
                write_contents(&mut reader)
            }
            BuilderFileReader::Bytes(mut bytes) => write_contents(&mut bytes),
            BuilderFileReader::Reader(reader) => write_contents(reader),
        }
    }

    fn apply_file_entry_patch<WriterType: Write + Seek>(
        mut writer: WriterType,
        patch_offset: u64,
        patch_info: FilePatchInfo,
    ) -> Result<()> {
        writer.seek(SeekFrom::Start(patch_offset))?;
        writer.write_u64(patch_info.filesize)?;
        writer.write_u32(patch_info.crc)?;
        Ok(())
    }
}
