use crate::binary::BinaryWriter;
use crate::{addon_metadata::AddonMetadata, result::Result, AddonTag, AddonType, Error, IDENT};
use crc::Crc;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::{
    fs::File,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

//Defaults
const DEFAULT_VERSION: u8 = 3;
const DEFAULT_STEAMID: u64 = 0;
const DEFAULT_DESCRIPTION: &str = "";
const DEFAULT_AUTHOR: &str = "unknown";
const DEFAULT_COMPRESSION: bool = false;

enum BuilderFileReader {
    FSFile(BufReader<File>),
    Bytes(Vec<u8>),
    Reader(Box<dyn Read>),
}

struct BuilderFile {
    filename: String,
    reader: BuilderFileReader,
}

struct FilePatchInfo {
    filesize: u64,
    crc: u32,
}

/// GMA File Builder.
///
/// The only required fields are 'name' and 'addon_tag'
pub struct GMABuilder {
    version: Option<u8>,
    steamid: Option<u64>,
    timestamp: Option<u64>,
    name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    files: Vec<BuilderFile>,
    addon_type: AddonType,
    addon_tags: [Option<AddonTag>; 2],
    compression: Option<bool>,
}

impl GMABuilder {
    /// Creates a new gma builder
    pub fn new() -> Self {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::new(0, 0))
            .as_secs() as u64;

        Self {
            version: Some(DEFAULT_VERSION),
            steamid: Some(DEFAULT_STEAMID),
            timestamp: Some(current_timestamp),
            name: None,
            description: Some(DEFAULT_DESCRIPTION.to_owned()),
            author: Some(DEFAULT_AUTHOR.to_owned()),
            files: Vec::new(),
            addon_type: AddonType::Tool,
            addon_tags: [None; 2],
            compression: Some(DEFAULT_COMPRESSION),
        }
    }

    /// Sets the gma version of the archive. Default : 3
    pub fn version(&mut self, version: u8) -> &mut Self {
        self.version = Some(version);
        self
    }

    /// Sets the steamid of the author. Default : 0
    pub fn steamid(&mut self, steamid: u64) -> &mut Self {
        self.steamid = Some(steamid);
        self
    }

    /// Sets the timestamp. Default : current time
    pub fn timestamp(&mut self, timestamp: u64) -> &mut Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Sets the name of the addon. Required
    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the addon. Default : ''
    pub fn description<S: Into<String>>(&mut self, description: S) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the name of the author. Default : 'unknown'
    pub fn author<S: Into<String>>(&mut self, author: S) -> &mut Self {
        self.author = Some(author.into());
        self
    }

    /// Enables or disables lzma compression. Default : false
    ///
    /// Garry's mod doesnt open compressed gma files.
    /// Support for compressed files is mostly here to interact with files downloaded straight
    /// from the steamworkshop that could be compressed
    pub fn compression(&mut self, c: bool) -> &mut Self {
        self.compression = Some(c);
        self
    }

    /// Sets the addon type. Required
    pub fn addon_type(&mut self, addon_type: AddonType) -> &mut Self {
        self.addon_type = addon_type;
        self
    }

    /// Adds tag to the addon.
    /// Only 2 tags are allowed at any given time, adding more will replace the oldest one
    pub fn addon_tag(&mut self, addon_tag: AddonTag) -> &mut Self {
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
    pub fn file_from_path<S: AsRef<Path>>(
        &mut self,
        path: S,
    ) -> std::result::Result<&mut Self, std::io::Error> {
        let file = File::open(&path)?;
        self.files.push(BuilderFile {
            filename: path.as_ref().to_string_lossy().as_ref().to_owned(),
            reader: BuilderFileReader::FSFile(BufReader::new(file)),
        });
        Ok(self)
    }

    pub fn file_with_name<P: AsRef<Path>, N: Into<String>>(
        &mut self,
        path: P,
        name: N,
    ) -> std::result::Result<&mut Self, std::io::Error> {
        let file = File::open(&path)?;
        self.files.push(BuilderFile {
            filename: name.into(),
            reader: BuilderFileReader::FSFile(BufReader::new(file)),
        });
        Ok(self)
    }

    /// Adds a file with the given filename and contents
    pub fn file_from_bytes<S: Into<String>>(&mut self, filename: S, bytes: Vec<u8>) -> &mut Self {
        self.files.push(BuilderFile {
            filename: filename.into(),
            reader: BuilderFileReader::Bytes(bytes),
        });
        self
    }

    /// Adds a file with the given filename and contents are read from `reader`
    pub fn file_from_reader<S: Into<String>, R: Read + 'static>(
        &mut self,
        filename: S,
        reader: R,
    ) -> &mut Self {
        self.files.push(BuilderFile {
            filename: filename.into(),
            reader: BuilderFileReader::Reader(Box::new(reader)),
        });
        self
    }

    /// Consumes the builder and writes the gma file contents to the given `writer`
    pub fn write_to<WriterType>(self, mut writer: WriterType) -> Result<()>
    where
        WriterType: Write + Seek,
    {
        match self.compression.unwrap() {
            true => {
                let buffer = Vec::with_capacity(1024 * 1024 * 32);
                let mut bufwriter = Cursor::new(buffer);
                Self::write_to_gen(self, &mut bufwriter)?;
                bufwriter.seek(SeekFrom::Start(0))?;
                lzma_rs::lzma_compress(&mut bufwriter, &mut writer).unwrap();
                Ok(())
            }
            false => Self::write_to_gen(self, writer),
        }
    }

    fn write_to_gen<WriterType: Write + Seek>(self, mut writer: WriterType) -> Result<()> {
        let name = self
            .name
            .expect("You need to provided a name for the addon file");

        Self::write_ident(&mut writer)?;
        //write version
        writer.write_u8(self.version.unwrap())?;
        //write steamid
        writer.write_u64(self.steamid.unwrap())?;
        //write timestamp
        writer.write_u64(self.timestamp.unwrap())?;
        //write required contents
        //this is unused right now so just write an empty string
        writer.write_u8(0)?;
        //write addon name
        writer.write_c_string(&name)?;
        //write metadata string
        let tags: Vec<AddonTag> = self
            .addon_tags
            .iter()
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect();
        let metadata = AddonMetadata::new(
            name.to_owned(),
            self.description.unwrap(),
            &self.addon_type,
            &tags,
        );
        let metadata_json = metadata.to_json();
        writer.write_c_string(&metadata_json)?;
        //write author name
        writer.write_c_string(&self.author.unwrap())?;
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
            let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
            let mut digest = crc.digest();
            loop {
                let read_result = reader.read(&mut buffer);
                match read_result {
                    Ok(0) => {
                        return Ok((
                            bytes_written,
                            FilePatchInfo {
                                filesize: bytes_written as u64,
                                crc: digest.finalize() as u32,
                            },
                        ));
                    }
                    Ok(n) => {
                        let data_slice = &buffer[0..n];
                        digest.update(data_slice);
                        writer.write_all(data_slice)?;
                        bytes_written += n;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(Error::IOError(e)),
                }
            }
        };
        match bfile.reader {
            BuilderFileReader::FSFile(mut reader) => write_contents(&mut reader),
            BuilderFileReader::Bytes(bytes) => write_contents(&mut bytes.as_slice()),
            BuilderFileReader::Reader(mut reader) => write_contents(&mut reader),
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
