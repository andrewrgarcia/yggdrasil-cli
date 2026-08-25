use ignore::WalkBuilder;

use crate::cli::Args;
use crate::types::FileEntry;

use super::filters::matches_filters;
use super::normalize::normalize_pattern;
use super::patterns::load_patterns_file;
use super::stdin::read_multiline_stdin;

use std::fs;

/// Does any selection pattern name a dot-path?
///
/// `--white`/`--only` are explicit: if the user typed `.windsurf/rules/x.md`,
/// the walker hiding dotfiles means the file can never be reached no matter
/// what they wrote. Naming a hidden path IS the request to see it.
///
/// Deliberately narrow — it looks for a leading dot on a path *component*, so
/// `--only src` keeps hiding `src/.cache`, and only a pattern that actually
/// mentions a dot-path lifts the filter.
fn mentions_hidden(patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        p.split('/')
            .any(|part| part.starts_with('.') && part != "." && part != "..")
    })
}

fn build_walker(args: &Args, force_hidden: bool) -> ignore::Walk {
    WalkBuilder::new(&args.dir)
        .hidden(!args.hidden && !force_hidden)
        .ignore(!args.no_ignore)
        .git_ignore(!args.no_ignore)
        .git_global(!args.no_ignore)
        .git_exclude(!args.no_ignore)
        .parents(!args.no_ignore)
        .filter_entry(|e| e.file_name() != ".git")
        .build()
}

/// Collect all file paths according to ignore/only filters and flags.
pub fn collect_files(args: &Args) -> Vec<FileEntry> {
    let mut ignore_patterns: Vec<String> =
        args.ignore.iter().map(|s| normalize_pattern(s)).collect();
    let mut only_patterns: Vec<String> = args.only.iter().map(|s| normalize_pattern(s)).collect();

    // --black
    if let Some(black_opt) = &args.black {
        match black_opt {
            Some(file) => ignore_patterns.extend(load_patterns_file(file)),
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

    // Resolved after --white/--black are loaded, so patterns that arrived
    // from a manifest file count too.
    let force_hidden = mentions_hidden(&only_patterns);

    let mut files = Vec::new();

    // Walk directory tree
    for entry in build_walker(args, force_hidden).filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }

        let path = entry.path().to_string_lossy().to_string();

        // --show <ext>
        if !args.show.is_empty() {
            let ext = entry
                .path()
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

        // Read file once
        let contents = fs::read_to_string(&path).unwrap_or_default();

        let line_count = contents.lines().count();
        let word_count = contents.split_whitespace().count();
        let token_est = ((word_count as f32) * 1.33).round() as usize;

        files.push(FileEntry {
            path,
            line_count,
            word_count,
            token_est,
        });
    }

    files
}