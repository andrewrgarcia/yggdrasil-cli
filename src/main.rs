use clap::Parser;
use walkdir::WalkDir;

mod formatter;
use formatter::{MarkdownFormatter, CliFormatter, OutputFormatter};
use std::fs::File;
use std::io::{self, Write};
use atty::Stream;

#[derive(Parser, Debug)]
#[command(author, version, about = "✨ Yggdrasil CLI — the god-tree of your codebase. AI-ready directory discovery.", long_about = None)]
struct Args {
    /// Root directory to scan
    #[arg(default_value = ".")]
    dir: String,

    /// Show only files with these extensions (e.g. --show tex rs md)
    #[arg(long, num_args = 0.., value_delimiter = ' ')]
    show: Vec<String>,


    /// Print file contents as well
    #[arg(long)]
    contents: bool,

    /// Output in Markdown format
    #[arg(long)]
    md: bool,

    /// Restrict output to these files/dirs/globs
    #[arg(long, num_args = 1.., value_delimiter = ' ')]
    only: Vec<String>,

    /// Provide inline patterns to ignore (globs, names, etc.)
    #[arg(long, num_args = 1.., value_delimiter = ' ')]
    ignore: Vec<String>,

    /// Load blacklist patterns from a file (like .gitignore)
    #[arg(long)]
    blacklist: Option<String>,

    /// Write output to file instead of stdout
    #[arg(long)]
    out: Option<String>,
}

fn matches_ignore_filters(path: &str, filters: &Vec<String>) -> bool {
    if filters.is_empty() {
        return false;
    }

    let norm_path = path.strip_prefix("./").unwrap_or(path);
    let base = std::path::Path::new(norm_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for f in filters {
        let norm_filter = f.strip_prefix("./").unwrap_or(f);

        if norm_path == norm_filter || base == norm_filter {
            return true;
        }

        if norm_path.starts_with(norm_filter) {
            return true;
        }

        if let Ok(pattern) = glob::Pattern::new(norm_filter) {
            if pattern.matches(norm_path) || pattern.matches(base) {
                return true;
            }
        }
    }
    false
}


fn matches_only_filters(path: &str, filters: &Vec<String>) -> bool {
    if filters.is_empty() {
        return true;
    }

    // Normalize path (remove leading ./ if present)
    let norm_path = path.strip_prefix("./").unwrap_or(path);

    // Get basename for file-only checks
    let base = std::path::Path::new(norm_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for f in filters {
        // Normalize filter too
        let norm_filter = f.strip_prefix("./").unwrap_or(f);

        // Exact match (full path or basename)
        if norm_path == norm_filter || base == norm_filter {
            return true;
        }

        // Directory prefix (src matches src/main.rs)
        if norm_path.starts_with(norm_filter) {
            return true;
        }

        // Glob pattern
        if let Ok(pattern) = glob::Pattern::new(norm_filter) {
            if pattern.matches(norm_path) || pattern.matches(base) {
                return true;
            }
        }
    }
    false
}

fn load_ignore_file(path: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            patterns.push(line.to_string());
        }
    }
    patterns
}


/// Collect files according to filters
fn collect_files(root: &str, args: &Args, ignore_patterns: &Vec<String>) -> Vec<String> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path().to_string_lossy().to_string();

            // Extension filtering (--show)
            if !args.show.is_empty() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if !args.show.contains(&ext.to_string()) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Exclude if matches ignore (inline or from blacklist file)
            if matches_ignore_filters(&path, ignore_patterns) {
                continue;
            }

            // Apply --only
            if !matches_only_filters(&path, &args.only) {
                continue;
            }

            files.push(path);
        }
    }

    files
}


fn run(
    fmt: &dyn OutputFormatter,
    args: &Args,
    root: &str,
    ignore_patterns: &Vec<String>,
    out: &mut dyn std::io::Write,
) {
    let files = collect_files(root, args, ignore_patterns);

    fmt.print_preamble(root, out);
    fmt.print_index(&files, out);

    if args.contents {
        fmt.print_contents(&files, out);
    }
}

fn main() {
    let args = Args::parse();
    let root = args.dir.clone();

    let mut ignore_patterns = args.ignore.clone();
    if let Some(file) = &args.blacklist {
        let from_file = load_ignore_file(file);
        ignore_patterns.extend(from_file);
        ignore_patterns.push(file.clone());
    }

    // Prepare output sink
    let mut writer: Box<dyn Write> = if let Some(out_file) = &args.out {
        // auto-ignore output file
        ignore_patterns.push(out_file.clone());
        Box::new(File::create(out_file).expect("Failed to create output file"))
    } else {
        Box::new(io::stdout())
    };

    if args.md {
        let fmt = MarkdownFormatter;
        run(&fmt, &args, &root, &ignore_patterns, &mut *writer);
    } else {
        // colored only if stdout is a tty
        let fmt = CliFormatter { colored: args.out.is_none() && atty::is(Stream::Stdout) };
        run(&fmt, &args, &root, &ignore_patterns, &mut *writer);
    }
}



