use clap::{Parser};

#[derive(Parser, Debug)]
#[command(
    name = "ygg",
    author,
    version,
    about = "✨ Yggdrasil CLI — the god-tree of your codebase.",
    long_about = "Flatten your project into an AI-ready snapshot codex — index + contents in one command.",
    after_help = "
Examples:
  # Export repo contents as Markdown-coded codex
  ygg --show --contents --out SHOW.md

  # List all Rust files (paths only)
  ygg --show rs

  # List all JSON files except node_modules & .next
  ygg --show json --ignore node_modules .next

  # Restrict scan to Markdown files only inside src/ dir
  ygg --show md --only src

  # Exclude files via blacklist
  ygg --show --blacklist BLACK.md --contents 

  # Show only files listed in a manifest and export as CLI-coded codex
  ygg --show --manifest WHITE.md --contents --out show.txt
"
)]
pub struct Args {
    /// Root directory to scan
    #[arg(default_value = ".")]
    pub dir: String,

    /// Show only files with these extensions (e.g. --show tex rs md)
    #[arg(long, num_args = 0.., value_delimiter = ' ')]
    pub show: Vec<String>,

    /// Print file contents as well
    #[arg(long)]
    pub contents: bool,

    /// Output in Markdown format
    #[arg(long)]
    pub md: bool,

    /// Restrict output to these files/dirs/globs
    #[arg(long, num_args = 1.., value_delimiter = ' ')]
    pub only: Vec<String>,

    /// Provide inline patterns to ignore (globs, names, etc.)
    #[arg(long, num_args = 1.., value_delimiter = ' ')]
    pub ignore: Vec<String>,

    /// Load blacklist patterns from a file (like .gitignore)
    #[arg(long)]
    pub blacklist: Option<String>,

    /// Load manifest (explicit file list to show)
    #[arg(long)]
    pub manifest: Option<String>,

    /// Write output to file instead of stdout
    #[arg(long)]
    pub out: Option<String>,
}
