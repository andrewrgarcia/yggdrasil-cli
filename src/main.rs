//! main.rs — with args.rs fully absorbed

mod diff;
mod formatters;
mod pick;
mod scanner;
mod snapshot;
mod sniff;
mod tree;
mod types;

use clap::{Parser, Subcommand};
use diff::run_diff;
use snapshot::run_snapshot;

#[derive(Parser, Debug)]
#[command(
    name = "ygg",
    author,
    version,
    about = "✨ Yggdrasil CLI — the god-tree of your codebase.",
    long_about = "Flatten your project into an AI-ready snapshot codex — index + contents in one command.\n\nRun `ygg` bare for a token-weighted listing of the current directory, or `ygg tree` for the full tree."
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

    /// Render the directory tree, weighted by token cost
    ///
    /// Every directory reports the total tokens of its whole subtree, even
    /// when collapsed — so you can see what will actually fit in a context
    /// window before you build a codex.
    Tree {
        /// Root path to render
        #[arg(default_value = ".")]
        path: String,

        /// Levels to expand below the root (0 = root's children only)
        #[arg(long, short = 'L')]
        depth: Option<usize>,

        /// Include hidden files and directories
        #[arg(long, short = 'a')]
        all: bool,

        /// Do not honour .gitignore / .ignore files
        #[arg(long)]
        no_ignore: bool,

        /// Show directories only
        #[arg(long)]
        dirs_only: bool,

        /// Hide the token column
        #[arg(long)]
        no_stats: bool,

        /// Show each file's declared symbols (fn / struct / class / …)
        #[arg(long, short = 's')]
        symbols: bool,

        /// Cap symbols listed per file (default 10)
        #[arg(long, value_name = "N")]
        max_symbols: Option<usize>,

        /// Exclude these paths/globs from the tree
        #[arg(long, num_args = 1.., value_delimiter = ' ')]
        ignore: Vec<String>,
    },

    /// Interactively pick files in a tree UI and write a codex
    ///
    /// Expand folders, tick files or whole subtrees, and watch the token
    /// cost of your selection live — then write straight to SHOW.md.
    Pick {
        /// Root path to pick from
        #[arg(default_value = ".")]
        path: String,

        /// Include hidden files and directories
        #[arg(long, short = 'a')]
        all: bool,

        /// Do not honour .gitignore / .ignore files
        #[arg(long)]
        no_ignore: bool,

        /// Output file (default SHOW.md)
        #[arg(long)]
        out: Option<String>,
    },
}

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Root directory to scan
    #[arg(long, default_value = ".")]
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
    /// `--tree` is a direct alias of `--only`
    #[arg(long, alias = "tree", num_args = 1.., value_delimiter = ' ')]
    pub only: Vec<String>,

    /// Do not display line counts in file index
    #[arg(long)]
    pub no_lines: bool,

    /// Provide inline patterns to ignore (globs, names, etc.)
    #[arg(long, num_args = 1.., value_delimiter = ' ')]
    pub ignore: Vec<String>,

    /// Include hidden (dot) files in the scan
    #[arg(long)]
    pub hidden: bool,

    /// Do not honour .gitignore / .ignore files while scanning
    #[arg(long)]
    pub no_ignore: bool,

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
    /// Example: `ygg --printed` or `ygg --printed MyFile.md`
    #[arg(long, short = 'p', num_args = 0..=1)]
    pub printed: Option<Option<String>>,

    /// Interactive index-only export to SHOW.md
    /// Equivalent to: interactive `--white` + markdown output, but no FILES section
    #[arg(long, num_args = 0..=1)]
    pub treed: Option<Option<String>>,

    /// Split output into token-bounded packets (K = thousands, default 32)
    #[arg(long, num_args = 0..=1, value_name = "K")]
    pub split: Option<Option<usize>>,

    /// Package generated file output into a ZIP archive
    #[arg(long)]
    pub zip: bool,

    /// Write output to file instead of stdout
    #[arg(long)]
    pub out: Option<String>,

    /// Align diff tags to a fixed column
    #[arg(long)]
    pub align_tags: bool,

    /// Expand an entry file into its full local dependency set via static
    /// import analysis. Resolves imports recursively, bounded to --dir.
    /// Feeds discovered files into the snapshot pipeline exactly like --only.
    ///
    /// Example:
    ///   ygg --sniff src/main.py --printed
    ///   ygg --sniff scripts/audit.py --dir ../my-project --printed --split
    #[arg(long)]
    pub sniff: Option<String>,
}

pub mod cli {
    pub use super::{Args, Cli, Commands};
}

fn main() {
    // viceroy: bare `ygg` used to print --help. It now lists the current
    // directory, weighted by token cost. `ygg --help` still prints help.
    if std::env::args().len() == 1 {
        tree::run_list(".");
        return;
    }

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Diff {
            from,
            to,
            align_tags,
        }) => {
            run_diff(from, to, align_tags);
        }

        Some(Commands::Tree {
            path,
            depth,
            all,
            no_ignore,
            dirs_only,
            no_stats,
            symbols,
            max_symbols,
            ignore,
        }) => {
            tree::run_tree(tree::TreeRequest {
                path,
                depth,
                all,
                no_ignore,
                dirs_only,
                no_stats,
                symbols,
                max_symbols,
                ignore,
            });
        }

        Some(Commands::Pick {
            path,
            all,
            no_ignore,
            out,
        }) => {
            pick::run_pick(&path, all, no_ignore, out);
        }

        None => {
            run_snapshot(cli.args);
        }
    }
}