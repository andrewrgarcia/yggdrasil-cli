use crate::cli::Args;
use crate::snapshot::{render_snapshot, run_snapshot};
use crate::tree::scan::{scan_tree, ScanOpts};

use super::clipboard::{self, Route};
use super::state::PickState;
use super::ui::{run_picker, Outcome};

/// Build the `Args` the picker's selection implies.
///
/// The picker fills the same two slots the flags would have: the selection is
/// `--only <paths…>`, and the destination is either `--printed` (write) or
/// nothing at all (copy, which renders to memory).
fn args_for(path: &str, all: bool, no_ignore: bool, only: Vec<String>, printed: Option<Option<String>>) -> Args {
    Args {
        dir: path.to_string(),
        show: Vec::new(),
        // For the write path this is resolved by apply_output_plan; for the
        // copy path nothing resolves it, so it must already be true.
        contents: printed.is_none(),
        md: printed.is_none(),
        only,
        no_lines: false,
        ignore: Vec::new(),
        hidden: all,
        no_ignore,
        black: None,
        white: None,
        whited: None,
        printed,
        treed: None,
        split: None,
        zip: false,
        out: None,
        align_tags: false,
        sniff: None,
    }
}

/// `ygg pick` — scan, choose interactively, then write or copy the codex.
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
        eprintln!("⚠️  nothing to pick under '{path}'");
        return;
    }

    let mut state = PickState::new(tree);

    match run_picker(&mut state) {
        Err(e) => eprintln!("⚠️  picker failed: {e}"),

        // User quit; say nothing, like ctrl-c anywhere else.
        Ok(Outcome::Quit) => {}

        Ok(Outcome::Write(paths)) => {
            let count = paths.len();
            let target = out.clone().unwrap_or_else(|| "SHOW.md".to_string());

            run_snapshot(args_for(path, all, no_ignore, paths, Some(out)));

            eprintln!("🌳 {count} file{} → {target}", plural(count));
        }

        Ok(Outcome::Copy(paths)) => {
            let count = paths.len();
            let codex = render_snapshot(&args_for(path, all, no_ignore, paths, None));
            let text = String::from_utf8_lossy(&codex);

            let kb = text.len() / 1024;

            match clipboard::copy(&text) {
                Ok(Route::Tool(name)) => {
                    eprintln!(
                        "📋 {count} file{} · {kb} KB → clipboard ({name})",
                        plural(count)
                    );
                }

                // The escape sequence was written, but many terminals drop it
                // without a word and there is no reply to read. Claiming
                // success here would be a guess, so say what actually
                // happened and leave a file behind either way.
                Ok(Route::Osc52) => {
                    let target = rescue(&codex, &out);
                    eprintln!(
                        "📋 {count} file{} · {kb} KB sent via terminal escape — \
                         your terminal may ignore it.",
                        plural(count)
                    );
                    eprintln!("   For reliable copying, {}.", clipboard::install_hint());
                    if let Some(t) = target {
                        eprintln!("   Wrote {t} as a fallback.");
                    }
                }

                Err(e) => {
                    let target = rescue(&codex, &out);
                    eprintln!("⚠️  {e}");
                    if let Some(t) = target {
                        eprintln!("   Wrote {t} instead, so the selection is not lost.");
                    }
                }
            }
        }
    }
}

/// Write the rendered codex to disk when the clipboard could not be trusted.
///
/// Losing a hand-built selection because a clipboard tool was missing is the
/// one outcome worth engineering against.
fn rescue(codex: &[u8], out: &Option<String>) -> Option<String> {
    let target = out.clone().unwrap_or_else(|| "SHOW.md".to_string());
    match std::fs::write(&target, codex) {
        Ok(()) => Some(target),
        Err(_) => None,
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}