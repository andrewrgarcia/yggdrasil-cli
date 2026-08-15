//! Tree mode — the directory view, weighted by token cost.
//!
//! This is deliberately NOT an `ls` clone. The column that matters is
//! `tokens`, aggregated up every directory, because that is the number
//! that decides what fits in a model's context window. `--symbols` goes
//! one further: it shows what each file *declares*, which is the thing
//! you actually want when deciding whether to include it in a codex.

pub mod node;
pub mod render;
pub mod run;
pub mod scan;

pub use node::{RawEntry, Stats, TreeNode};
pub use render::{human_tokens, render_list, render_tree, RenderOpts};
pub use run::{run_list, run_tree, TreeRequest};
pub use scan::{scan_tree, ScanOpts};