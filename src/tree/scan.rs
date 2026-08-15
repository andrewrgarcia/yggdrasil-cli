use std::path::{Component, Path, PathBuf};

use ignore::WalkBuilder;

use crate::scanner::filters::matches_filters;

use super::node::{build_tree, RawEntry, Stats, TreeNode};

#[derive(Debug, Clone, Default)]
pub struct ScanOpts {
    /// Include dotfiles.
    pub hidden: bool,
    /// Do not honour .gitignore / .ignore / global excludes.
    pub no_ignore: bool,
    /// Render directories only.
    pub dirs_only: bool,
    /// Extra exclusion patterns (same semantics as --ignore).
    pub ignore_patterns: Vec<String>,
}

fn file_stats(path: &Path) -> Stats {
    match std::fs::read_to_string(path) {
        Ok(text) => Stats::from_text(&text),
        // Binary or unreadable: it exists, it just has no token weight.
        Err(_) => Stats::default(),
    }
}

fn root_display_name(root: &Path, raw: &str) -> String {
    std::fs::canonicalize(root)
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| raw.to_string())
}

/// Walk `root` fully and return an aggregated tree.
///
/// The walk is always full-depth even when the caller intends to render only
/// a few levels: a collapsed directory still reports its true subtree token
/// cost, which is the entire point of the view.
pub fn scan_tree(root: &str, opts: &ScanOpts) -> TreeNode {
    let root_path = PathBuf::from(root);

    let walker = WalkBuilder::new(&root_path)
        .hidden(!opts.hidden)
        .ignore(!opts.no_ignore)
        .git_ignore(!opts.no_ignore)
        .git_global(!opts.no_ignore)
        .git_exclude(!opts.no_ignore)
        .parents(!opts.no_ignore)
        // .git is never interesting, even under --all.
        .filter_entry(|e| e.file_name() != ".git")
        .build();

    let mut entries: Vec<RawEntry> = Vec::new();

    for dent in walker.filter_map(|e| e.ok()) {
        if dent.depth() == 0 {
            continue; // the root itself
        }

        let is_dir = dent.file_type().map(|t| t.is_dir()).unwrap_or(false);

        if opts.dirs_only && !is_dir {
            continue;
        }

        let path = dent.path();

        if !opts.ignore_patterns.is_empty()
            && matches_filters(&path.to_string_lossy(), &opts.ignore_patterns, false)
        {
            continue;
        }

        let Ok(rel) = path.strip_prefix(&root_path) else {
            continue;
        };

        let comps: Vec<String> = rel
            .components()
            .filter_map(|c| match c {
                Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                _ => None,
            })
            .collect();

        if comps.is_empty() {
            continue;
        }

        let stats = if is_dir {
            Stats::default()
        } else {
            file_stats(path)
        };

        entries.push(RawEntry {
            rel: comps,
            is_dir,
            stats,
        });
    }

    build_tree(root_display_name(&root_path, root), &root_path, entries)
}