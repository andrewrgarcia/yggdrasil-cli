mod formatter;

use clap::Parser;
use formatter::{CliFormatter, MarkdownFormatter, OutputFormatter};
use walkdir::WalkDir;

/// 🌲 Yggdrasil CLI – the god-tree of your codebase
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Output as Markdown
    #[arg(long)]
    md: bool,

    /// Output as CLI with colors
    #[arg(long)]
    cli: bool,

    /// Only show files with these extensions (e.g. --show tex rs c)
    #[arg(long, num_args = 1..)]
    show: Option<Vec<String>>,

    /// Show contents of files
    #[arg(long)]
    contents: bool,

    /// Root directory (default = current dir)
    dir: Option<String>,
}

fn main() {
    let args = Args::parse();
    let root = args.dir.as_deref().unwrap_or(".").to_string();

    if args.md {
        let fmt = MarkdownFormatter {};
        run(&fmt, &args, &root);
    } else {
        let fmt = CliFormatter {};
        run(&fmt, &args, &root);
    }
}


fn run<F: OutputFormatter>(fmt: &F, args: &Args, root: &str) {
    fmt.print_preamble(root);

    let mut files = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let path = entry.path().display().to_string();

            // --- Extension filter ---
            if let Some(exts) = &args.show {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if !exts.contains(&ext.to_string()) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            files.push(path);
        }
    }

    fmt.print_index(&files);

    if args.contents {
        fmt.print_contents(&files);
    }
}
