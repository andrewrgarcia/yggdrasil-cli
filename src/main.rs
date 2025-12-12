//! main.rs — with args.rs fully absorbed

mod types;
mod snapshot;
mod scanner;
mod formatters;
mod diff;

use clap::{Parser, Subcommand, CommandFactory};
use snapshot::run_snapshot;
use diff::run_diff;


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
    pub args: Args,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compare two sets of files (original vs modified)
    Diff {
        /// Source files or directories
        #[arg(required = true)]
        from: Vec<String>,
        /// Target files or directories
        #[arg(required = true)]
        to: Vec<String>,
        /// Align diff tags to a fixed column
        #[arg(long)]
        align_tags: bool,
    },
}

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

    /// Load blacklist patterns (like .gitignore) or enter manually.
    #[arg(long, alias = "blacklist", num_args = 0..=1)]
    pub black: Option<Option<String>>,

    /// Load manifest (explicit file list to show) or enter manually.
    #[arg(long, alias = "manifest", num_args = 0..=1)]
    pub white: Option<Option<String>>,

    /// Shortcut for `--white --contents --out SHOW.md`
    /// Example: `ygg --whited summary.md`
    #[arg(long, num_args = 0..=1)]
    pub whited: Option<Option<String>>,

    /// Shortcut for `--contents --out SHOW.md`
    /// Example: `ygg printed` or `ygg printed MyFile.md`
    #[arg(long, short = 'p', num_args = 0..=1)]
    pub printed: Option<Option<String>>,

    /// Split output into token-bounded packets (K = thousands, default 32)
    #[arg(long, num_args = 0..=1, value_name = "K")]
    pub split: Option<Option<usize>>,

    /// Write output to file instead of stdout
    #[arg(long)]
    pub out: Option<String>,

    /// Align diff tags to a fixed column
    #[arg(long)]
    pub align_tags: bool,
}


pub mod cli {
    pub use super::{Cli, Args, Commands};
}


fn main() {
    let cli = Cli::parse();

    if std::env::args().len() == 1 {
        Cli::command().print_help().unwrap();
        println!();
        return;
    }

    match cli.command {
        Some(Commands::Diff { from, to, align_tags }) => {
            run_diff(from, to, align_tags);
        }

        None => {
            run_snapshot(cli.args);
        }
    }
}

