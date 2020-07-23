#[cfg(test)]
mod test {
    use gma::{AddonTag, AddonType, GMABuilder};
    use std::io::Cursor;

    #[test]
    fn build_parse_gma() {
        const VERSION: u8 = 3;
        const STEAMID: u64 = 123456;
        const TIMESTAMP: u64 = 987654;
        const NAME: &str = "ADDON_NAME";
        const DESC: &str = "ADDON_DESC";
        const AUTHOR: &str = "AUTHOR_NAME";
        const TYPE: AddonType = AddonType::Model;
        const TAG1: AddonTag = AddonTag::Build;
        const TAG2: AddonTag = AddonTag::Fun;
        const ENTRY_NAME: &str = "file1";
        const ENTRY_DATA: &[u8] = b"hello";

        let mut buffer: Vec<u8> = Vec::new();

        GMABuilder::new()
            .version(VERSION)
            .steamid(STEAMID)
            .timestamp(TIMESTAMP)
            .name(NAME)
            .description(DESC)
            .addon_type(TYPE)
            .addon_tag(TAG1)
            .addon_tag(TAG2)
            .author(AUTHOR)
            .file_from_bytes(ENTRY_NAME, ENTRY_DATA)
            .write_to(Cursor::new(&mut buffer))
            .unwrap();

        let archive = gma::load_from_memory(&buffer).unwrap();
        assert_eq!(archive.version(), VERSION);
        assert_eq!(archive.author_steamid(), STEAMID);
        assert_eq!(archive.timestamp(), TIMESTAMP);
        assert_eq!(archive.name(), NAME);
        assert_eq!(archive.description(), DESC);
        assert_eq!(archive.addon_type().unwrap(), TYPE);
        assert!(archive.contains_tag(TAG1));
        assert!(archive.contains_tag(TAG2));
        //this fields isnt currently used and is hardcoded to this
        assert_eq!(archive.author(), AUTHOR);

        let entry = archive
            .entries()
            .next()
            .expect("Archive should countain one entry");
        assert_eq!(entry.filename(), ENTRY_NAME);
        assert_eq!(entry.size(), ENTRY_DATA.len() as u64);
        assert_eq!(entry.crc(), 0907060870);
        archive
            .read_entry(entry, |_, reader| {
                let mut entry_buffer = Vec::new();
                reader.read_to_end(&mut entry_buffer).unwrap();
                assert_eq!(entry_buffer.as_slice(), ENTRY_DATA);
            })
            .unwrap();
    }
}
