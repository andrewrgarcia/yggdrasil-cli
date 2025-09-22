use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ygg",
    author,
    version,
    about = "✨ Yggdrasil CLI — the god-tree of your codebase.",
    long_about = "Flatten your project into an AI-ready snapshot codex — index + contents in one command."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub args: Args, // your global flags stay the same
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compare two sets of files (original vs modified)
    Diff {
        /// First set of files (before `--`)
        #[arg(required = true, num_args = 1.., value_delimiter = ' ')]
        from: Vec<String>,

        /// Second set of files (after `--`)
        #[arg(required = true, num_args = 1.., last = true)]
        to: Vec<String>,
    },
}

// Your existing Args struct remains unchanged
#[derive(clap::Args, Debug)]
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

    /// Do not display line counts in file index
    #[arg(long)]
    pub no_lines: bool,

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
