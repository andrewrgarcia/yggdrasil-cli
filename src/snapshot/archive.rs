use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Name of the index at the root of a tree archive.
///
/// Not `README.md`: a packed repo usually already has one, and the leading
/// underscore also sorts this above every source folder so it is the first
/// thing a reader — human or model — meets.
const INDEX_NAME: &str = "_CODEX.md";
/// Used only if the selection somehow already contains `_CODEX.md`.
const INDEX_FALLBACK: &str = "_CODEX.ygg.md";

/// Marker that opens the inline-contents half of a codex.
const FILES_MARKER: &str = "\n## FILES";

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
/// simple and immediately readable by LLM upload interfaces. This is the
/// shape `--split` wants: N shard documents, no source tree.
pub fn write_zip_archive(
    archive_path: &Path,
    entries: &[(String, Vec<u8>)],
) -> zip::result::ZipResult<()> {
    let file = File::create(archive_path)?;
    let mut archive = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    for (entry_name, contents) in entries {
        archive.start_file(entry_name.as_str(), options)?;
        archive.write_all(contents)?;
    }

    archive.finish()?;
    Ok(())
}

// ── tree archives ─────────────────────────────────────────────────────
//
// The other shape: not one flattened blob but the real files at their real
// paths, under an index. Upload interfaces unpack an archive and index each
// member separately, so keeping the tree intact buys per-file retrieval and
// correct extensions — while the index carries the map and the token bill
// that a bare folder of files would lose.

/// What actually made it into a tree archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeArchiveReport {
    /// Where the index landed — `_CODEX.md` unless that name was taken.
    pub index_name: String,
    pub written: usize,
    /// Paths that could not be read; reported, never silently lost.
    pub skipped: Vec<String>,
}

/// Convert a collected path into a safe archive entry name.
///
/// Entries must be relative and must not climb: `./src/a.rs` and
/// `../proj/src/a.rs` both flatten their leading dots away, so an archive
/// built with `--dir ..` still unpacks into one sane folder instead of
/// escaping the extraction directory.
pub fn entry_name(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|c| !c.is_empty() && *c != "." && *c != "..")
        .collect::<Vec<_>>()
        .join("/")
}

/// Turn a rendered codex into the archive's index document.
///
/// The preamble and INDEX are kept verbatim — including the word/token
/// counts, which were computed over the *full* codex and so still describe
/// the whole selection. Only the inline FILES section goes, because those
/// bytes now exist as real files beside this document.
pub fn readme_from_codex(codex: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(codex);

    let head = match text.find(FILES_MARKER) {
        Some(i) => &text[..i + 1],
        None => &text[..],
    };

    let mut out = head.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(
        "\n> 📦 Packed as a folder: every path in the INDEX above is a real \
         file in this archive.\n",
    );
    out.into_bytes()
}

/// Write files as a directory-preserving archive, index first.
pub fn write_tree_archive(
    archive_path: &Path,
    index_md: &[u8],
    files: &[String],
) -> zip::result::ZipResult<TreeArchiveReport> {
    let names: Vec<String> = files.iter().map(|p| entry_name(p)).collect();

    let taken: HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
    let index_name = if taken.contains(INDEX_NAME) {
        INDEX_FALLBACK
    } else {
        INDEX_NAME
    }
    .to_string();

    let file = File::create(archive_path)?;
    let mut archive = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    // Index first, so it is the first entry an unpacker lists.
    archive.start_file(index_name.as_str(), options)?;
    archive.write_all(index_md)?;

    let mut written = 0usize;
    let mut skipped = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for (path, name) in files.iter().zip(names.iter()) {
        // Two source paths can normalise to one entry name; a duplicate
        // entry makes some unzippers refuse the whole archive.
        if name.is_empty() || !seen.insert(name.clone()) {
            continue;
        }

        match fs::read(path) {
            Ok(bytes) => {
                archive.start_file(name.as_str(), options)?;
                archive.write_all(&bytes)?;
                written += 1;
            }
            Err(_) => skipped.push(path.clone()),
        }
    }

    archive.finish()?;

    Ok(TreeArchiveReport {
        index_name,
        written,
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODEX: &str = "\
# YGG CODEX
project: yggdrasil-cli

> ✍️ Words: 5447
> 🪙 Tokens (est.): 7245

## INDEX
path          : lines | words | tokens
./src/main.rs :   262 |   871 |   1158
total_loc: 262

## FILES
=== FILE_BEGIN ===
path=./src/main.rs
--- CONTENT ---
fn main() {}
=== FILE_END ===
";

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

    #[test]
    fn entry_names_are_relative_and_cannot_climb() {
        assert_eq!(entry_name("./src/pick/ui.rs"), "src/pick/ui.rs");
        assert_eq!(entry_name("../proj/src/a.rs"), "proj/src/a.rs");
        assert_eq!(entry_name("src\\pick\\ui.rs"), "src/pick/ui.rs");
        assert_eq!(entry_name("/etc/passwd"), "etc/passwd");
    }

    #[test]
    fn index_keeps_the_table_and_drops_the_contents() {
        let got = String::from_utf8(readme_from_codex(CODEX.as_bytes())).unwrap();

        assert!(got.contains("## INDEX"));
        assert!(got.contains("./src/main.rs :   262 |   871 |   1158"));
        assert!(got.contains("total_loc: 262"));
        // The point of the whole exercise: no inline file bodies.
        assert!(!got.contains("## FILES"), "got: {got}");
        assert!(!got.contains("FILE_BEGIN"));
    }

    #[test]
    fn index_reports_the_whole_selections_token_bill() {
        // Stats were computed over the full codex; trimming must not touch them.
        let got = String::from_utf8(readme_from_codex(CODEX.as_bytes())).unwrap();
        assert!(got.contains("Tokens (est.): 7245"));
    }

    #[test]
    fn an_index_only_codex_survives_untouched() {
        let src = "# YGG CODEX\n\n## INDEX\npath\ntotal_loc: 0\n";
        let got = String::from_utf8(readme_from_codex(src.as_bytes())).unwrap();
        assert!(got.starts_with(src));
    }

    #[test]
    fn a_repo_readme_never_collides_with_the_index() {
        let dir = std::env::temp_dir().join("ygg_zip_readme");
        let _ = fs::create_dir_all(&dir);
        let readme_src = dir.join("README.md");
        fs::write(&readme_src, b"project readme").unwrap();

        let archive = dir.join("out.zip");
        let report = write_tree_archive(
            &archive,
            b"# index",
            &[readme_src.to_string_lossy().to_string()],
        )
        .unwrap();

        assert_eq!(report.index_name, "_CODEX.md");
        assert_eq!(report.written, 1);
        assert!(report.skipped.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unreadable_paths_are_reported_not_swallowed() {
        let archive = std::env::temp_dir().join("ygg_zip_missing.zip");
        let report = write_tree_archive(
            &archive,
            b"# index",
            &["definitely/not/a/real/file.rs".to_string()],
        )
        .unwrap();

        assert_eq!(report.written, 0);
        assert_eq!(report.skipped.len(), 1);

        let _ = fs::remove_file(&archive);
    }

    /// A private scratch directory, cleared on entry so a crashed run does
    /// not poison the next one.
    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ygg_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Every `.rs` file under `dir`, depth-first, in sorted order.
    fn walk_rs(dir: &Path, out: &mut Vec<String>) {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        entries.sort();

        for path in entries {
            if path.is_dir() {
                walk_rs(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path.to_string_lossy().to_string());
            }
        }
    }

    /// The archive is only worth anything if what comes out is what went in.
    ///
    /// Packs this crate's own sources — real files, real nesting, real
    /// non-ASCII (the codex is full of emoji and box-drawing) — plus a blob
    /// covering every byte value, then reads the archive back and compares.
    #[test]
    fn packed_files_survive_the_round_trip_byte_for_byte() {
        use std::io::Read;

        let mut files = Vec::new();
        walk_rs(Path::new("src"), &mut files);
        assert!(
            files.len() > 5,
            "expected to find this crate's own sources; cwd is {:?}",
            std::env::current_dir()
        );

        let scratch = scratch_dir("roundtrip");

        // Every byte value, including NUL and invalid UTF-8 — proves entries
        // are stored as bytes and never pass through a String.
        let blob = scratch.join("all-bytes.dat");
        let all_bytes: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
        fs::write(&blob, &all_bytes).unwrap();
        files.push(blob.to_string_lossy().to_string());

        let archive_path = scratch.join("out.zip");
        let index = b"# YGG CODEX\n\n## INDEX\npath\ntotal_loc: 0\n".to_vec();

        let report = write_tree_archive(&archive_path, &index, &files).unwrap();
        assert_eq!(report.written, files.len());
        assert!(report.skipped.is_empty(), "skipped: {:?}", report.skipped);

        let mut zip = zip::ZipArchive::new(File::open(&archive_path).unwrap()).unwrap();
        assert_eq!(
            zip.len(),
            files.len() + 1,
            "one entry per file, plus the index"
        );

        {
            let mut entry = zip.by_name(&report.index_name).unwrap();
            let mut got = Vec::new();
            entry.read_to_end(&mut got).unwrap();
            assert_eq!(got, index, "index document was altered");
        }

        for path in &files {
            let name = entry_name(path);

            let mut entry = zip
                .by_name(&name)
                .unwrap_or_else(|e| panic!("missing entry {name}: {e}"));

            let mut got = Vec::new();
            entry.read_to_end(&mut got).unwrap();

            let want = fs::read(path).unwrap();

            assert_eq!(got.len(), want.len(), "size differs for {name}");
            // Compared as a whole rather than printed: a mismatch here would
            // dump a source file into the test output.
            assert!(got == want, "bytes differ for {name}");
        }

        let _ = fs::remove_dir_all(&scratch);
    }

    /// The reason paths are kept instead of basenames: this crate has three
    /// files called `mod.rs`, and flattening would collapse them into one.
    #[test]
    fn same_named_files_in_different_dirs_stay_distinct() {
        use std::io::Read;

        let mut files = Vec::new();
        walk_rs(Path::new("src"), &mut files);

        let mods: Vec<&String> = files.iter().filter(|p| p.ends_with("mod.rs")).collect();
        assert!(mods.len() >= 2, "expected several mod.rs files: {mods:?}");

        let scratch = scratch_dir("distinct");
        let archive_path = scratch.join("out.zip");

        let picked: Vec<String> = mods.iter().map(|p| (*p).clone()).collect();
        let report = write_tree_archive(&archive_path, b"# index", &picked).unwrap();
        assert_eq!(report.written, picked.len(), "entries were collapsed");

        let mut zip = zip::ZipArchive::new(File::open(&archive_path).unwrap()).unwrap();

        for path in &picked {
            let name = entry_name(path);
            assert!(name.contains('/'), "entry lost its directory: {name}");

            let mut entry = zip.by_name(&name).unwrap();
            let mut got = Vec::new();
            entry.read_to_end(&mut got).unwrap();
            assert_eq!(got, fs::read(path).unwrap(), "{name}");
        }

        let _ = fs::remove_dir_all(&scratch);
    }
}