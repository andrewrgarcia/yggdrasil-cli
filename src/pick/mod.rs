//! Pick mode — interactive codex building.
//!
//! `ygg pick` opens the token-weighted tree in the terminal: expand folders,
//! tick files or whole subtrees, watch the context bill update live, then
//! write the selection straight through the normal snapshot pipeline.

pub mod run;
pub mod state;
pub mod ui;

pub use run::run_pick;
