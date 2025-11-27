use std::fs::File;
use std::io::{self, Write};
use crate::cli::Args;

/// The writer can be:
/// - Memory(Vec<u8>): buffer everything so we can inject metadata later
/// - Stdout: direct streaming
/// - File: (unused now; Markdown always uses Memory)
#[allow(dead_code)]
pub enum OutputTarget {
    Memory(Vec<u8>),
    Stdout,
    File(File),
}

/// Open the correct writer target
///
/// RULE:
///     - If --out <file>  → use Memory buffer (inject tokens later)
///     - If no --out      → write directly to stdout
pub fn open_writer(args: &Args) -> OutputTarget {
    if let Some(_) = &args.out {
        // Markdown to file → we must buffer → injection patch applied later
        OutputTarget::Memory(Vec::new())
    } else {
        // Default: stdout
        OutputTarget::Stdout
    }
}

// Implement Write for OutputTarget so snapshot routines can write normally
impl Write for OutputTarget {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutputTarget::Memory(inner) => {
                inner.extend_from_slice(buf);
                Ok(buf.len())
            }
            OutputTarget::Stdout => {
                io::stdout().write(buf)
            }
            OutputTarget::File(file) => {
                file.write(buf)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutputTarget::Memory(_) => Ok(()),
            OutputTarget::Stdout => io::stdout().flush(),
            OutputTarget::File(file) => file.flush(),
        }
    }
}
