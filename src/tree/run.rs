use atty::Stream;

use super::render::{render_list, render_tree, RenderOpts};
use super::scan::{scan_tree, ScanOpts};

/// `extract_symbols` hard-caps its own output at 10; this matches it.
const DEFAULT_MAX_SYMBOLS: usize = 10;

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
        show_symbols: false,
    };

    render_list(&tree, &opts, &mut std::io::stdout());
}

/// Everything `ygg tree` accepts, in one bag.
///
// viceroy: replaces a seven-positional-argument run_tree(). Adding --symbols
// would have made it eight, all of them bool.
#[derive(Debug, Clone)]
pub struct TreeRequest {
    pub path: String,
    pub depth: Option<usize>,
    pub all: bool,
    pub no_ignore: bool,
    pub dirs_only: bool,
    pub no_stats: bool,
    pub symbols: bool,
    pub max_symbols: Option<usize>,
    pub ignore: Vec<String>,
}

/// `ygg tree` — the box-drawn view.
pub fn run_tree(req: TreeRequest) {
    // --symbols is about files; asking for both is a contradiction, and
    // silently printing an empty tree would be worse than saying so.
    if req.symbols && req.dirs_only {
        eprintln!("⚠️  --symbols has no effect with --dirs-only; ignoring --symbols.");
    }

    let want_symbols = req.symbols && !req.dirs_only;

    let scan_opts = ScanOpts {
        hidden: req.all,
        no_ignore: req.no_ignore,
        dirs_only: req.dirs_only,
        symbols: want_symbols,
        max_symbols: req.max_symbols.unwrap_or(DEFAULT_MAX_SYMBOLS),
        ignore_patterns: req.ignore,
    };

    let tree = scan_tree(&req.path, &scan_opts);

    let opts = RenderOpts {
        max_depth: req.depth,
        colored: colored_stdout(),
        show_stats: !req.no_stats,
        show_symbols: want_symbols,
    };

    render_tree(&tree, &opts, &mut std::io::stdout());
}