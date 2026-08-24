//! Picker state — everything about *what* is shown and selected, and nothing
//! about how it is drawn. Pure and synchronous so it can be tested without a
//! terminal.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::tree::node::TreeNode;

/// Selection state of a row's checkbox.
///
/// Directories are derived, never stored: a dir is `All` when every file in
/// its subtree is selected, `Partial` when some are, `None` otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    None,
    Partial,
    All,
}

/// One visible line of the picker, flattened out of the tree.
///
/// Owned (not borrowed from the tree) so the UI loop can hold rows while
/// mutating expansion/selection between frames.
#[derive(Debug, Clone)]
pub struct Row {
    pub path: PathBuf,
    /// `display_name()` — dirs carry their trailing slash.
    pub name: String,
    pub is_dir: bool,
    /// Box-drawing stems *including* the `├── ` / `└── ` connector.
    pub prefix: String,
    pub tokens: usize,
    /// Dirs only; 0 for files.
    pub file_count: usize,
    pub expanded: bool,
    pub mark: Mark,
}

pub struct PickState {
    pub root: TreeNode,
    /// Paths of expanded directories.
    pub expanded: HashSet<PathBuf>,
    /// Paths of selected *files*. Dir marks are derived from this.
    pub selected: HashSet<PathBuf>,
    pub cursor: usize,
    pub scroll: usize,
}

impl PickState {
    pub fn new(root: TreeNode) -> Self {
        let mut expanded = HashSet::new();
        // Open the first level so the picker never starts as a single line.
        for child in &root.children {
            if child.is_dir {
                expanded.insert(child.path.clone());
                break; // just the first dir; the rest stay collapsed
            }
        }
        PickState {
            root,
            expanded,
            selected: HashSet::new(),
            cursor: 0,
            scroll: 0,
        }
    }

    // ── row flattening ────────────────────────────────────────────────

    /// Flatten the tree into the currently visible rows, in draw order.
    pub fn rows(&self) -> Vec<Row> {
        let mut out = Vec::new();
        self.walk(&self.root, "", &mut out);
        out
    }

    fn walk(&self, node: &TreeNode, prefix: &str, out: &mut Vec<Row>) {
        let total = node.children.len();

        for (i, child) in node.children.iter().enumerate() {
            let is_last = i + 1 == total;
            let connector = if is_last { "└── " } else { "├── " };
            let expanded = child.is_dir && self.expanded.contains(&child.path);

            out.push(Row {
                path: child.path.clone(),
                name: child.display_name(),
                is_dir: child.is_dir,
                prefix: format!("{prefix}{connector}"),
                tokens: child.stats.tokens,
                file_count: child.file_count,
                expanded,
                mark: self.mark_of(child),
            });

            if expanded {
                let next = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                self.walk(child, &next, out);
            }
        }
    }

    fn mark_of(&self, node: &TreeNode) -> Mark {
        if !node.is_dir {
            return if self.selected.contains(&node.path) {
                Mark::All
            } else {
                Mark::None
            };
        }
        let picked = count_selected(node, &self.selected);
        if picked == 0 || node.file_count == 0 {
            Mark::None
        } else if picked == node.file_count {
            Mark::All
        } else {
            Mark::Partial
        }
    }

    // ── mutations ─────────────────────────────────────────────────────

    /// Toggle selection at `path`. Files flip; directories flip their whole
    /// subtree — everything on unless everything already was, then off.
    pub fn toggle(&mut self, path: &PathBuf) {
        let Some(node) = find(&self.root, path) else {
            return;
        };

        if !node.is_dir {
            if !self.selected.remove(path) {
                self.selected.insert(path.clone());
            }
            return;
        }

        let files = subtree_files(node);
        let all_on = !files.is_empty() && files.iter().all(|f| self.selected.contains(f));

        for f in files {
            if all_on {
                self.selected.remove(&f);
            } else {
                self.selected.insert(f);
            }
        }
    }

    /// Select every file, or clear the selection if everything already is.
    pub fn toggle_all(&mut self) {
        let files = subtree_files(&self.root);
        let all_on = !files.is_empty() && files.iter().all(|f| self.selected.contains(f));
        if all_on {
            self.selected.clear();
        } else {
            self.selected.extend(files);
        }
    }

    pub fn set_expanded(&mut self, path: &PathBuf, open: bool) {
        if open {
            self.expanded.insert(path.clone());
        } else {
            self.expanded.remove(path);
        }
    }

    pub fn toggle_expanded(&mut self, path: &PathBuf) {
        if !self.expanded.remove(path) {
            self.expanded.insert(path.clone());
        }
    }

    /// Open every directory in the tree.
    pub fn expand_all(&mut self) {
        fn rec(node: &TreeNode, set: &mut HashSet<PathBuf>) {
            for child in &node.children {
                if child.is_dir {
                    set.insert(child.path.clone());
                    rec(child, set);
                }
            }
        }
        rec(&self.root, &mut self.expanded);
    }

    // ── totals ────────────────────────────────────────────────────────

    /// (selected file count, summed token estimate) — the live context bill.
    pub fn selection_cost(&self) -> (usize, usize) {
        let mut count = 0usize;
        let mut tokens = 0usize;
        fn rec(node: &TreeNode, sel: &HashSet<PathBuf>, count: &mut usize, tokens: &mut usize) {
            for child in &node.children {
                if child.is_dir {
                    rec(child, sel, count, tokens);
                } else if sel.contains(&child.path) {
                    *count += 1;
                    *tokens += child.stats.tokens;
                }
            }
        }
        rec(&self.root, &self.selected, &mut count, &mut tokens);
        (count, tokens)
    }

    /// Selected file paths in deterministic (sorted) order, ready to feed
    /// into the snapshot pipeline as `--only` values.
    pub fn selection_paths(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .selected
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        v.sort();
        v
    }
}

// ── free helpers ──────────────────────────────────────────────────────

fn find<'a>(node: &'a TreeNode, path: &PathBuf) -> Option<&'a TreeNode> {
    if &node.path == path {
        return Some(node);
    }
    node.children.iter().find_map(|c| find(c, path))
}

fn subtree_files(node: &TreeNode) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn rec(node: &TreeNode, out: &mut Vec<PathBuf>) {
        for child in &node.children {
            if child.is_dir {
                rec(child, out);
            } else {
                out.push(child.path.clone());
            }
        }
    }
    rec(node, &mut out);
    out
}

fn count_selected(node: &TreeNode, selected: &HashSet<PathBuf>) -> usize {
    let mut n = 0;
    for child in &node.children {
        if child.is_dir {
            n += count_selected(child, selected);
        } else if selected.contains(&child.path) {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::node::{build_tree, RawEntry, Stats};
    use std::path::Path;

    fn file(rel: &[&str], tokens: usize) -> RawEntry {
        RawEntry {
            rel: rel.iter().map(|s| s.to_string()).collect(),
            is_dir: false,
            stats: Stats {
                lines: 1,
                words: tokens,
                tokens,
            },
            ..Default::default()
        }
    }

    fn dir(rel: &[&str]) -> RawEntry {
        RawEntry {
            rel: rel.iter().map(|s| s.to_string()).collect(),
            is_dir: true,
            ..Default::default()
        }
    }

    fn state() -> PickState {
        let root = build_tree(
            "proj".into(),
            Path::new("."),
            vec![
                dir(&["src"]),
                dir(&["src", "sniff"]),
                file(&["src", "sniff", "python.rs"], 724),
                file(&["src", "sniff", "resolve.rs"], 645),
                file(&["src", "main.rs"], 686),
                file(&["README.md"], 1718),
            ],
        );
        let mut s = PickState::new(root);
        s.expanded.clear(); // tests control expansion explicitly
        s
    }

    #[test]
    fn collapsed_tree_shows_only_top_level() {
        let s = state();
        let names: Vec<String> = s.rows().iter().map(|r| r.name.clone()).collect();
        assert_eq!(names, vec!["src/", "README.md"]);
    }

    #[test]
    fn expanding_reveals_children_in_place() {
        let mut s = state();
        s.toggle_expanded(&PathBuf::from("./src"));
        let names: Vec<String> = s.rows().iter().map(|r| r.name.clone()).collect();
        assert_eq!(names, vec!["src/", "sniff/", "main.rs", "README.md"]);

        s.toggle_expanded(&PathBuf::from("./src/sniff"));
        let names: Vec<String> = s.rows().iter().map(|r| r.name.clone()).collect();
        assert_eq!(
            names,
            vec![
                "src/",
                "sniff/",
                "python.rs",
                "resolve.rs",
                "main.rs",
                "README.md"
            ]
        );
    }

    #[test]
    fn stems_continue_the_parent_pipe() {
        let mut s = state();
        s.expand_all();
        let rows = s.rows();
        let python = rows.iter().find(|r| r.name == "python.rs").unwrap();
        // src/ is not the last root child, so its descendants carry `│`.
        assert!(python.prefix.starts_with("│   "), "got: {:?}", python.prefix);
    }

    #[test]
    fn dir_toggle_selects_whole_subtree() {
        let mut s = state();
        s.toggle(&PathBuf::from("./src"));
        let (count, tokens) = s.selection_cost();
        assert_eq!(count, 3);
        assert_eq!(tokens, 724 + 645 + 686);

        // Second toggle clears it.
        s.toggle(&PathBuf::from("./src"));
        assert_eq!(s.selection_cost(), (0, 0));
    }

    #[test]
    fn partial_selection_marks_ancestors_partial() {
        let mut s = state();
        s.expand_all();
        s.toggle(&PathBuf::from("./src/sniff/python.rs"));

        let rows = s.rows();
        let by_name = |n: &str| rows.iter().find(|r| r.name == n).unwrap().mark;

        assert_eq!(by_name("python.rs"), Mark::All);
        assert_eq!(by_name("resolve.rs"), Mark::None);
        assert_eq!(by_name("sniff/"), Mark::Partial);
        assert_eq!(by_name("src/"), Mark::Partial);
    }

    #[test]
    fn selecting_the_missing_file_completes_the_dir() {
        let mut s = state();
        s.toggle(&PathBuf::from("./src/sniff/python.rs"));
        s.toggle(&PathBuf::from("./src/sniff/resolve.rs"));
        s.expand_all();
        let rows = s.rows();
        let sniff = rows.iter().find(|r| r.name == "sniff/").unwrap();
        assert_eq!(sniff.mark, Mark::All);
    }

    #[test]
    fn toggle_all_flips_everything() {
        let mut s = state();
        s.toggle_all();
        assert_eq!(s.selection_cost().0, 4);
        s.toggle_all();
        assert_eq!(s.selection_cost().0, 0);
    }

    #[test]
    fn selection_paths_are_sorted_files_only() {
        let mut s = state();
        s.toggle(&PathBuf::from("./src/sniff"));
        assert_eq!(
            s.selection_paths(),
            vec!["./src/sniff/python.rs", "./src/sniff/resolve.rs"]
        );
    }

    #[test]
    fn empty_dir_never_reports_all() {
        let root = build_tree("proj".into(), Path::new("."), vec![dir(&["empty"])]);
        let s = PickState::new(root);
        assert_eq!(s.rows()[0].mark, Mark::None);
    }
}
