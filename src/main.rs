use clap::Parser;
use walkdir::WalkDir;
use glob::Pattern;

mod formatter;
use formatter::{MarkdownFormatter, CliFormatter, OutputFormatter};

#[derive(Parser, Debug)]
#[command(author, version, about = "✨ Yggdrasil CLI — the god-tree of your codebase. AI-ready directory discovery.", long_about = None)]
struct Args {
    /// Root directory to scan
    #[arg(default_value = ".")]
    dir: String,

    /// Show only files with these extensions (e.g. --show tex rs md)
    #[arg(long, num_args = 1.., value_delimiter = ' ')]
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
}

/// Check if a path matches any --only filters
fn matches_only_filters(path: &str, filters: &Vec<String>) -> bool {
    if filters.is_empty() {
        return true; // no restriction
    }

    for f in filters {
        // Exact match
        if path == f {
            return true;
        }

        // Directory prefix match
        if path.starts_with(f) {
            return true;
        }

        // Glob pattern (*.tex, src/*.rs, etc.)
        if let Ok(pattern) = Pattern::new(f) {
            if pattern.matches(path) {
                return true;
            }
        }
    }
    false
}

/// Collect files according to filters
fn collect_files(root: &str, args: &Args) -> Vec<String> {
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

            // Apply --only
            if !matches_only_filters(&path, &args.only) {
                continue;
            }

            files.push(path);
        }
    }

    files
}

fn run(formatter: &dyn OutputFormatter, args: &Args, root: &str) {
    let files = collect_files(root, args);
    formatter.print_preamble(root);
    formatter.print_index(&files);
    if args.contents {
        formatter.print_contents(&files);
    }
}

fn main() {
    let args = Args::parse();
    let root = args.dir.clone();

    if args.md {
        let fmt = MarkdownFormatter;
        run(&fmt, &args, &root);
    } else {
        let fmt = CliFormatter;
        run(&fmt, &args, &root);
    }
}
