/// This example shows how to read a gma file and print out some information about it
use gma;
fn main() {
    let archive = gma::open("myfile.gma").unwrap();
    println!("Version : {}", archive.version());
    println!("Author steam id : {}", archive.author_steamid());
    println!("Timestamp : {}", archive.timestamp());
    println!("Name : {}", archive.name());
    println!("Description : {}", archive.description());
    println!("Addon Type : {:?}", archive.addon_type());
    println!("Addon Type : {:?}", archive.addon_tags());
    println!("Author name : {}", archive.author());
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
}
