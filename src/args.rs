use crate::{Deflate, ZipDeflate};
pub use clap::Parser;
use std::path::Path;

#[derive(Parser)]
#[command(version = "0.1.0")]
pub struct Args {
    pub zipfile_name: String,
    #[arg(
        short = 'f',
        long = "follow",
        default_value_t = false,
        help = "whether to follow symlink"
    )]
    pub follow_symlink: bool,
    pub filelist: Vec<String>,
}

pub fn run(args: Args) {
    let mut lists = Vec::new();
    match args.follow_symlink {
        true => {
            args.filelist.iter().for_each(|f| {
                lists.append(crate::scan_symlink_follow(Path::new(f)).as_mut().unwrap())
            });
        }
        false => {
            args.filelist
                .iter()
                .for_each(|f| lists.append(crate::scan_symlink(Path::new(f)).as_mut().unwrap()));
        }
    };
    let mut deflate = ZipDeflate::new(Path::new(&args.zipfile_name));
    deflate.write_archive(&lists);
    deflate.finish().unwrap();
}
