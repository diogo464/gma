# GMA
![Rust](https://github.com/diogo464/gma/workflows/Rust/badge.svg)
[![Crates.io version](https://img.shields.io/crates/v/gma.svg)](https://crates.io/crates/gma)

[Documentation](https://docs.rs/gma)

A crate to read and write to .gma files.

Reading/Writing lzma compressed files is supported.
Garry's mod cant read compressed gma files but some when downloaded directly from the steam workshop some files are lzma compressed.

## Reading a .gma file
```rust
    let archive = gma::open("myfile.gma").unwrap();
    println!("Version : {}", archive.version());
    println!("Author steam id : {}", archive.author_steamid());
    println!("Timestamp : {}", archive.timestamp());
    println!("Name : {}", archive.name());
    println!("Description : {}", archive.description());
    println!("Addon Type : {:?}", archive.addon_type());
    println!("Addon Type : {:?}", archive.addon_tags());
    println!("Author name : {}", archive.author());
    println!("Compressed : {}", archive.compressed());
    println!();

    for entry in archive.entries() {
        println!("{} :", entry.filename());
        println!("\tSize : {} bytes", entry.size());
        println!("\tCRC32 : {:x}", entry.crc());

        //Only print the contents of lua files
        if entry.filename().ends_with(".lua") {
            archive
                .read_entry(entry, |_, reader| {
                    let mut file_contents = String::new();
                    reader.read_to_string(&mut file_contents).unwrap();
                    println!("\tContents : '{}'", file_contents);
                })
                .expect("Error when reading the file");
        }
    }
```

## Creating a .gma file
```rust
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

    let mut builder = GMABuilder::new();

    builder
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
        .compression(true);

    builder.write_to(&mut writer).unwrap();
```