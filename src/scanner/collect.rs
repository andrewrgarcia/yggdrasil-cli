use walkdir::WalkDir;

use crate::cli::Args;
use crate::types::FileEntry;

use super::stdin::read_multiline_stdin;
use super::patterns::load_patterns_file;
use super::filters::matches_filters;
use super::counter::count_lines;

/// Collect all file paths according to ignore/only filters and flags.
pub fn collect_files(args: &Args) -> Vec<FileEntry> {
    let mut ignore_patterns = args.ignore.clone();
    let mut only_patterns = args.only.clone();

    // --black
    if let Some(black_opt) = &args.black {
        match black_opt {
            // --black file
            Some(file) => ignore_patterns.extend(load_patterns_file(file)),
            // --black (no argument, read from stdin)
            None => {
                if let Some(p) = read_multiline_stdin("Enter BLACK patterns (one per line):") {
                    ignore_patterns.extend(p);
                }
            }
        }
    }

    // --white
    if let Some(white_opt) = &args.white {
        match white_opt {
            Some(file) => only_patterns.extend(load_patterns_file(file)),
            None => {
                if let Some(p) = read_multiline_stdin("Enter WHITE patterns (one per line):") {
                    only_patterns.extend(p);
                }
            }
        }
    }

    let mut files = Vec::new();

    // Walk directory tree
    for entry in WalkDir::new(&args.dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path().to_string_lossy().to_string();

            // --show <ext>
            if !args.show.is_empty() {
                let ext = entry.path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                if !args.show.contains(&ext.to_string()) {
                    continue;
                }
            }

            // ignore filters
            if matches_filters(&path, &ignore_patterns, false) {
                continue;
            }

            // only filters
            if !matches_filters(&path, &only_patterns, true) {
                continue;
            }

            files.push(FileEntry {
                path: path.clone(),
                line_count: count_lines(&path),
            });
        }
    }

    files
}

