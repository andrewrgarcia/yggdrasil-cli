//! Pick mode — interactive codex building.
//!
//! `ygg pick` opens the token-weighted tree in the terminal: expand folders,
//! tick files or whole subtrees, watch the context bill update live, then
//! write the codex to disk (`w`), send it straight to the system clipboard
//! (`c`), or pack the selection as a browsable archive (`z`).

pub mod clipboard;
pub mod run;
pub mod state;
pub mod ui;

pub use run::run_pick;