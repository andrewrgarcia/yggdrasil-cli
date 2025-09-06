use clap::Parser;
use walkdir::WalkDir;

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
