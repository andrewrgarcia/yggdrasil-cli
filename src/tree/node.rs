use std::path::{Path, PathBuf};

/// Per-file (or aggregated per-directory) weight.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub lines: usize,
    pub words: usize,
    pub tokens: usize,
}

impl Stats {
    pub fn from_text(text: &str) -> Self {
        let lines = text.lines().count();
        let words = text.split_whitespace().count();
        // Same estimator collect_files uses; keep the two in lockstep.
        let tokens = ((words as f32) * 1.33).round() as usize;
        Stats {
            lines,
            words,
            tokens,
        }
    }

    pub fn merge(&mut self, other: &Stats) {
        self.lines += other.lines;
        self.words += other.words;
        self.tokens += other.tokens;
    }
}

/// A flat scan result, before it is folded into the tree.
///
/// `Default` is derived so construction sites (and tests) survive new fields.
#[derive(Debug, Clone, Default)]
pub struct RawEntry {
    /// Path components relative to the scan root.
    pub rel: Vec<String>,
    pub is_dir: bool,
    pub stats: Stats,
    /// Declared top-level symbols, when `--symbols` is on. Files only.
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    /// For files: own stats. For directories: aggregate of the whole subtree.
    pub stats: Stats,
    pub file_count: usize,
    pub dir_count: usize,
    /// Files only. Empty unless symbol extraction was requested.
    pub symbols: Vec<String>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(name: String, path: PathBuf, is_dir: bool) -> Self {
        TreeNode {
            name,
            path,
            is_dir,
            stats: Stats::default(),
            file_count: 0,
            dir_count: 0,
            symbols: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Name as rendered — directories carry a trailing slash.
    pub fn display_name(&self) -> String {
        if self.is_dir {
            format!("{}/", self.name)
        } else {
            self.name.clone()
        }
    }

    fn child_index(&mut self, name: &str, is_dir: bool) -> usize {
        if let Some(i) = self.children.iter().position(|c| c.name == name) {
            return i;
        }
        let path = self.path.join(name);
        self.children
            .push(TreeNode::new(name.to_string(), path, is_dir));
        self.children.len() - 1
    }

    fn insert(&mut self, comps: &[String], entry: &RawEntry) {
        let Some(name) = comps.first() else {
            return;
        };
        let is_leaf = comps.len() == 1;

        // Interior components are always directories; only the leaf knows.
        let idx = self.child_index(name, if is_leaf { entry.is_dir } else { true });

        if is_leaf {
            let child = &mut self.children[idx];
            child.is_dir = entry.is_dir;
            child.stats = entry.stats;
            child.symbols = entry.symbols.clone();
        } else {
            self.children[idx].is_dir = true;
            self.children[idx].insert(&comps[1..], entry);
        }
    }

    /// Roll subtree stats and counts up into every directory node.
    fn aggregate(&mut self) {
        if !self.is_dir {
            return;
        }

        let mut stats = Stats::default();
        let mut files = 0usize;
        let mut dirs = 0usize;

        for child in self.children.iter_mut() {
            child.aggregate();
            stats.merge(&child.stats);
            if child.is_dir {
                dirs += 1 + child.dir_count;
                files += child.file_count;
            } else {
                files += 1;
            }
        }

        self.stats = stats;
        self.file_count = files;
        self.dir_count = dirs;
    }

    /// Directories first, then natural (human) ordering by name.
    fn sort(&mut self) {
        self.children.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| natord::compare_ignore_case(&a.name, &b.name))
        });
        for child in self.children.iter_mut() {
            child.sort();
        }
    }
}

/// Fold a flat scan into a rooted, aggregated, sorted tree.
pub fn build_tree(root_name: String, root_path: &Path, entries: Vec<RawEntry>) -> TreeNode {
    let mut root = TreeNode::new(root_name, root_path.to_path_buf(), true);

    for entry in &entries {
        root.insert(&entry.rel, entry);
    }

    root.aggregate();
    root.sort();
    root
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn tree() -> TreeNode {
        build_tree(
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
        )
    }

    #[test]
    fn directories_aggregate_subtree_tokens() {
        let root = tree();
        assert_eq!(root.stats.tokens, 724 + 645 + 686 + 1718);

        let src = root.children.iter().find(|c| c.name == "src").unwrap();
        assert_eq!(src.stats.tokens, 724 + 645 + 686);
        assert_eq!(src.file_count, 3);
        assert_eq!(src.dir_count, 1);

        let sniff = src.children.iter().find(|c| c.name == "sniff").unwrap();
        assert_eq!(sniff.stats.tokens, 724 + 645);
    }

    #[test]
    fn directories_sort_before_files() {
        let root = tree();
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["src", "README.md"]);
    }

    #[test]
    fn implied_parents_become_directories() {
        // Only the leaf is declared; `a` and `b` must be inferred as dirs.
        let root = build_tree("r".into(), Path::new("."), vec![file(&["a", "b", "c.rs"], 10)]);
        let a = &root.children[0];
        assert!(a.is_dir);
        assert!(a.children[0].is_dir);
        assert!(!a.children[0].children[0].is_dir);
        assert_eq!(root.stats.tokens, 10);
    }

    #[test]
    fn symbols_survive_the_fold() {
        let entry = RawEntry {
            rel: vec!["src".into(), "lib.rs".into()],
            is_dir: false,
            symbols: vec!["parse".into(), "Config".into()],
            ..Default::default()
        };
        let root = build_tree("r".into(), Path::new("."), vec![entry]);
        let lib = &root.children[0].children[0];
        assert_eq!(lib.symbols, vec!["parse", "Config"]);
        // Directories never carry symbols.
        assert!(root.children[0].symbols.is_empty());
    }
}