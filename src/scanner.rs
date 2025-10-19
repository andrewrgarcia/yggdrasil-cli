use walkdir::WalkDir;
use glob::Pattern;
use crate::{args::Args, types::FileEntry};
use std::io::{self, Read};


pub fn read_multiline_stdin(prompt: &str) -> Option<Vec<String>> {
    use colored::Colorize;
    println!("{}", prompt.white());
    println!("{}", "💡 Tip: You can paste multiple file paths copied from VS Code (Right-Click → Copy Relative Path).".white());
    println!("{}", "↪ Finish with Ctrl+D (Linux/macOS) or Ctrl+Z then Enter (Windows).".white());
    println!("{}", "↪ Press Ctrl+C to cancel.".white());
    println!();

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).ok()?;
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        println!("⚠️ No content provided. Aborting.");
        None
    } else {
        Some(
            trimmed
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(String::from)
                .collect(),
        )
    }
}


pub fn load_patterns_file(path: &str) -> Vec<String> { 
    std::fs::read_to_string(path)
        .map(|c| {
            c.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn matches_filters(path: &str, filters: &[String], default: bool) -> bool {
    if filters.is_empty() {
        return default;
    }

    let norm_path = path.strip_prefix("./").unwrap_or(path);
    let base = std::path::Path::new(norm_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    filters.iter().any(|f| {
        let norm_filter = f.strip_prefix("./").unwrap_or(f);
        norm_path == norm_filter
            || base == norm_filter
            || norm_path.starts_with(norm_filter)
            || Pattern::new(norm_filter).map(|p| p.matches(norm_path) || p.matches(base)).unwrap_or(false)
    })
}

fn count_lines(path: &str) -> usize {
    std::fs::read_to_string(path).map(|c| c.lines().count()).unwrap_or(0)
}

pub fn collect_files(args: &Args) -> Vec<FileEntry> {
    let mut ignore_patterns = args.ignore.clone();
    let mut only_patterns = args.only.clone();

    // --- Handle black (ignore) patterns ---
    if let Some(black_opt) = &args.black {
        match black_opt {
            Some(file) => {
                ignore_patterns.extend(load_patterns_file(file));
            }
            None => {
                if let Some(patterns) = read_multiline_stdin("Enter BLACK patterns (one per line):") {
                    ignore_patterns.extend(patterns);
                }
            }
        }
    }

    // --- Handle white (manifest) patterns ---
    if let Some(white_opt) = &args.white {
        match white_opt {
            Some(file) => {
                only_patterns.extend(load_patterns_file(file));
            }
            None => {
                if let Some(patterns) = read_multiline_stdin("Enter WHITE patterns (one per line):") {
                    only_patterns.extend(patterns);
                }
            }
        }
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(&args.dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path().to_string_lossy().to_string();

            // Extension filter
            if !args.show.is_empty() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if !args.show.contains(&ext.to_string()) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if matches_filters(&path, &ignore_patterns, false) {
                continue;
            }
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_patterns_file_parses_correctly() {
        let tmpfile = "test_patterns.txt";
        std::fs::write(tmpfile, "foo\n#comment\nbar\n").unwrap();

        let patterns = load_patterns_file(tmpfile);
        assert_eq!(patterns, vec!["foo".to_string(), "bar".to_string()]);

        std::fs::remove_file(tmpfile).unwrap();
    }

    #[test]
    fn test_matches_filters_exact_and_glob() {
        let filters = vec!["src".to_string(), "*.rs".to_string()];

        assert!(matches_filters("src/main.rs", &filters, false));
        assert!(matches_filters("foo.rs", &filters, false));
        assert!(!matches_filters("docs/readme.md", &filters, false));
    }

    #[test]
    fn test_matches_filters_empty_returns_default() {
        let filters: Vec<String> = vec![];
        assert!(matches_filters("anything", &filters, true));
        assert!(!matches_filters("anything", &filters, false));
    }

    #[test]
    fn test_count_lines_simple_file() {
        let tmpfile = "test_count.txt";
        std::fs::write(tmpfile, "line1\nline2\nline3").unwrap();

        let count = count_lines(tmpfile);
        assert_eq!(count, 3);

        std::fs::remove_file(tmpfile).unwrap();
    }
}
