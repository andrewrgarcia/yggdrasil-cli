use std::fs::File;
use std::io::{self, Write};
use crate::args::Args;

pub fn open_writer(args: &Args) -> Box<dyn Write> {
    if let Some(out) = &args.out {
        Box::new(File::create(out).expect("Failed to write output file"))
    } else {
        Box::new(io::stdout())
    }
}

