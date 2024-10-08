use clannad::args;
use clannad::args::Parser;
use clannad::Args;

fn main() {
    let args = Args::parse();
    args::run(args);
}
