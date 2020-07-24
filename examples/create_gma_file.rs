use gma::{AddonTag, AddonType, GMABuilder};
use std::{fs::File, io::BufWriter};

fn main() {
    const VERSION: u8 = 3;
    const STEAMID: u64 = 123456;
    const TIMESTAMP: u64 = 987654;
    const NAME: &str = "ADDON_NAME";
    const DESC: &str = "ADDON_DESC";
    const AUTHOR: &str = "AUTHOR_NAME";
    const TYPE: AddonType = AddonType::Model;
    const TAG1: AddonTag = AddonTag::Build;
    const TAG2: AddonTag = AddonTag::Fun;

    let file = File::create("myaddon.gma").unwrap();
    let mut writer = BufWriter::new(file);

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
        .file_from_bytes("file1", b"hello")
        .compression(true)
        .write_to(&mut writer)
        .unwrap();
}
