//! Pick mode — interactive codex building.
//!
//! `ygg pick` opens the token-weighted tree in the terminal: expand folders,
//! tick files or whole subtrees, watch the context bill update live, then
//! either write the codex to disk (`w`) or send it straight to the system
//! clipboard (`c`) without ever touching a file.

pub mod clipboard;
pub mod run;
pub mod state;
pub mod ui;

pub use run::run_pick;
