use crate::cli::Args;
use crate::snapshot::run_snapshot;
use crate::tree::scan::{scan_tree, ScanOpts};

use super::state::PickState;
use super::ui::run_picker;

/// `ygg pick` — scan, choose interactively, then hand the chosen files to the
/// snapshot pipeline exactly as if they had been passed via `--only --printed`.
pub fn run_pick(path: &str, all: bool, no_ignore: bool, out: Option<String>) {
    let scan_opts = ScanOpts {
        hidden: all,
        no_ignore,
        dirs_only: false,
        symbols: false,
        max_symbols: 10,
        ignore_patterns: Vec::new(),
    };

    let tree = scan_tree(path, &scan_opts);

    if tree.file_count == 0 {
        eprintln!("⚠️  nothing to pick under '{}'", path);
        return;
    }

    let mut state = PickState::new(tree);

    match run_picker(&mut state) {
        Err(e) => eprintln!("⚠️  picker failed: {}", e),
        Ok(None) => {} // user quit; say nothing, like ctrl-c anywhere else
        Ok(Some(paths)) => {
            let count = paths.len();

            // The picker fills the same two slots the flags would have:
            // the selection is `--only <paths…>`, the output is `--printed`.
            let args = Args {
                dir: path.to_string(),
                show: Vec::new(),
                contents: false, // resolved by apply_output_plan via `printed`
                md: false,
                only: paths,
                no_lines: false,
                ignore: Vec::new(),
                hidden: all,
                no_ignore,
                black: None,
                white: None,
                whited: None,
                printed: Some(out),
                treed: None,
                split: None,
                zip: false,
                out: None,
                align_tags: false,
                sniff: None,
            };

            let target = args
                .printed
                .as_ref()
                .and_then(|o| o.clone())
                .unwrap_or_else(|| "SHOW.md".to_string());

            run_snapshot(args);

            eprintln!("🌳 {} file{} → {}", count, if count == 1 { "" } else { "s" }, target);
        }
    }
}
