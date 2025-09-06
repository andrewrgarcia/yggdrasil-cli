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

    /// Root directory to traverse (default = current directory)
    dir: Option<String>,
}

fn main() {
    let args = Args::parse();
    let root = args.dir.unwrap_or_else(|| ".".to_string());

    // Choose formatter
    if args.md {
        let fmt = MarkdownFormatter {};
        run(&fmt, &root);
    } else {
        let fmt = CliFormatter {};
        run(&fmt, &root);
    }
}

fn run<F: OutputFormatter>(fmt: &F, root: &str) {
    fmt.print_preamble(root);

    let mut files = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let path = entry.path().display().to_string();
            files.push(path);
        }
    }

    fmt.print_index(&files);
}
