use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Convert an output document path into its archive path.
///
/// SHOW.md -> SHOW.zip
/// report.txt -> report.zip
pub fn archive_path_for(output_path: &str) -> PathBuf {
    let mut archive_path = PathBuf::from(output_path);
    archive_path.set_extension("zip");
    archive_path
}

/// Package one or more generated documents into a single ZIP archive.
///
/// Entry names are stored without parent directories so the archive remains
/// simple and immediately readable by LLM upload interfaces.
pub fn write_zip_archive(
    archive_path: &Path,
    entries: &[(String, Vec<u8>)],
) -> zip::result::ZipResult<()> {
    let file = File::create(archive_path)?;
    let mut archive = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    for (entry_name, contents) in entries {
        archive.start_file(entry_name, options)?;
        archive.write_all(contents)?;
    }

    archive.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_document_extension_with_zip() {
        assert_eq!(
            archive_path_for("SHOW.md"),
            PathBuf::from("SHOW.zip")
        );

        assert_eq!(
            archive_path_for("output/project.txt"),
            PathBuf::from("output/project.zip")
        );
    }
}