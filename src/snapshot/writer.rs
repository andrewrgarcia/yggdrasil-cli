use std::fs::File;
use std::io::{self, Write};
use crate::args::Args;

/// Chooses stdout or file output, returns boxed writer.
pub fn open_writer(args: &Args) -> Box<dyn Write> {
    if let Some(out_file) = &args.out {
        Box::new(File::create(out_file).expect("Failed to create output file"))
    } else {
        Box::new(io::stdout())
    }
}
