#[cfg(test)]
mod tests {
    use gma::{AddonType, Tag};

    #[test]
    fn parse_genuine_gma() {
        let genuine = include_bytes!("genuine.gma");

        let archive = gma::load_from_memory(genuine).unwrap();

        assert_eq!(archive.version(), 3);
        assert_eq!(archive.author_steamid(), 0);
        assert_eq!(archive.timestamp(), 1595515015);
        assert_eq!(archive.name(), "My Test Addon");
        assert_eq!(archive.description(), "My Description");
        assert_eq!(archive.addon_type().unwrap(), AddonType::Gamemode);
        assert!(archive.contains_tag(Tag::Fun));
        assert!(archive.contains_tag(Tag::Cartoon));
        //this fields isnt currently used and is hardcoded to this
        assert_eq!(archive.author(), "Author Name");

        let entry = archive
            .entries()
            .next()
            .expect("Archive should countain one entry");
        assert_eq!(entry.filename(), "lua/hello.lua");
        assert_eq!(entry.size(), 3);
        //assert_eq!(entry.crc(), 0);
    }
}
