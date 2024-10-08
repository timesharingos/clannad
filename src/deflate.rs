use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::filter::FileInfo;
use crate::filter::FileType;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub trait Deflate {
    fn new(path: &Path) -> Self;
    fn write_dir(&mut self, dir: &Path);
    fn write_file(&mut self, file: &Path, content: &[u8]);
    fn write_symlink(&mut self, link: &Path, target: &Path);
    fn copy_dir(&mut self, src: &Path, dest: &Path);
    fn finish(self) -> Result<(), Box<dyn Error>>;

    fn write_archive(&mut self, filelist: &Vec<FileInfo>) {
        filelist.iter().for_each(|f| {
            println!("{}, {}", f.path, f.content_path);
            match &f.symlink_path {
                Some(points_to) => self.write_symlink(Path::new(&f.path), Path::new(&points_to)),
                None => match f.file_type {
                    FileType::DIRECTORY => {
                        //FIXME: if dest dir follows src, src will be dangling.
                        //
                        if f.path != f.content_path {
                            self.copy_dir(Path::new(&f.content_path), Path::new(&f.path));
                        }
                        self.write_dir(Path::new(&f.content_path));
                    }
                    FileType::REGULAR => self.write_file(
                        Path::new(&f.path),
                        fs::read_to_string(&f.content_path)
                            .expect(&format!("{} is not valid", f.content_path))
                            .as_bytes(),
                    ),
                    _ => unreachable!(),
                },
            };
        });
    }
}

/// Create Zip file using basic option.
pub struct ZipDeflate {
    writer: ZipWriter<File>,
}

impl Deflate for ZipDeflate {
    fn new(path: &Path) -> Self {
        Self {
            writer: ZipWriter::new(File::create(path).expect("archive file is not valid")),
        }
    }

    fn finish(self) -> Result<(), Box<dyn Error>> {
        self.writer.finish()?;
        Ok(())
    }

    fn write_dir(&mut self, dir: &Path) {
        match self
            .writer
            .add_directory_from_path(dir, SimpleFileOptions::default())
        {
            Ok(_) => {}
            Err(_) => println!("{}", format!("{} is illegal dir", dir.to_str().unwrap())),
        }
    }

    fn write_file(&mut self, file: &Path, content: &[u8]) {
        match self
            .writer
            .start_file_from_path(file, SimpleFileOptions::default())
        {
            Ok(_) => {}
            Err(_) => println!("{}", format!("{} is illegal path", file.to_str().unwrap())),
        }
        match self.writer.write_all(content) {
            Ok(_) => {}
            Err(_) => println!("{}", format!("{} is illegal file", file.to_str().unwrap())),
        };
    }

    fn write_symlink(&mut self, link: &Path, target: &Path) {
        match self
            .writer
            .add_symlink_from_path(link, target, SimpleFileOptions::default())
        {
            Ok(_) => {}
            Err(_) => println!(
                "{}",
                format!("{} is illegal symlink", link.to_str().unwrap())
            ),
        }
    }

    fn copy_dir(&mut self, src: &Path, dest: &Path) {
        match self.writer.deep_copy_file_from_path(src, dest) {
            Ok(_) => {}
            Err(_) => println!(
                "{}",
                format!(
                    "cannot copy {} to {}",
                    src.to_str().unwrap(),
                    dest.to_str().unwrap()
                )
            ),
        }
    }
}
