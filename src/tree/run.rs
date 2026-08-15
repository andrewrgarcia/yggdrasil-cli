use atty::Stream;

use super::render::{render_list, render_tree, RenderOpts};
use super::scan::{scan_tree, ScanOpts};

fn colored_stdout() -> bool {
    atty::is(Stream::Stdout)
}

/// Bare `ygg` — a flat, token-weighted listing of one directory.
pub fn run_list(path: &str) {
    let tree = scan_tree(path, &ScanOpts::default());

    let opts = RenderOpts {
        max_depth: Some(0),
        colored: colored_stdout(),
        show_stats: true,
    };

    render_list(&tree, &opts, &mut std::io::stdout());
}

/// `ygg tree` — the box-drawn view.
#[allow(clippy::too_many_arguments)]
pub fn run_tree(
    path: &str,
    depth: Option<usize>,
    all: bool,
    no_ignore: bool,
    dirs_only: bool,
    no_stats: bool,
    ignore_patterns: Vec<String>,
) {
    let scan_opts = ScanOpts {
        hidden: all,
        no_ignore,
        dirs_only,
        ignore_patterns,
    };

    let tree = scan_tree(path, &scan_opts);

    let opts = RenderOpts {
        max_depth: depth,
        colored: colored_stdout(),
        show_stats: !no_stats,
    };

    render_tree(&tree, &opts, &mut std::io::stdout());
}