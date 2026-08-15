//! Tree mode — the directory view, weighted by context cost.
//!
//! This is deliberately NOT an `ls` clone. The column that matters is
//! `tokens`, aggregated up every directory, because that is the number
//! that decides what fits in a model's context window.

pub mod node;
pub mod render;
pub mod run;
pub mod scan;

pub use node::{RawEntry, Stats, TreeNode};
pub use render::{human_tokens, render_list, render_tree, RenderOpts};
pub use run::{run_list, run_tree};
pub use scan::{scan_tree, ScanOpts};