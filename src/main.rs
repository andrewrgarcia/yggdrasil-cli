mod formatter;

use clap::Parser;
use formatter::{CliFormatter, MarkdownFormatter, OutputFormatter};

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

    // Default root directory
    let root = args.dir.unwrap_or_else(|| ".".to_string());

    // Choose formatter
    if args.md {
        let fmt = MarkdownFormatter {};
        fmt.print_preamble(&root);
    } else {
        let fmt = CliFormatter {};
        fmt.print_preamble(&root);
    }
}
