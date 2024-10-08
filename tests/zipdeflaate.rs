use clannad::{Deflate, Filter, SymlinkFilter, ZipDeflate};
use std::{
    fs::{remove_file, File},
    io::Read,
    path::Path,
};
use zip::ZipArchive;

#[test]
fn basic_deflate() {
    let mut deflate = ZipDeflate::new(Path::new("test.zip"));
    let mut filter = SymlinkFilter::new(Path::new("resources/normalfolder"));
    filter.scan();
    deflate.write_archive(filter.files().as_ref().expect("dir is valid"));
    deflate.finish().unwrap();

    let mut archive = ZipArchive::new(File::open("test.zip").unwrap()).unwrap();
    let mut content = String::new();
    archive
        .by_name("resources/normalfolder/level1/test1.ext1")
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();
    assert_eq!(content, String::from("123456"));

    remove_file("test.zip").unwrap();
}
