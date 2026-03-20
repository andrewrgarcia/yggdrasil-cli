use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::python::extract_python_imports;

/// The language-specific extraction strategy.
enum Extractor {
    Python,
    Unsupported,
}

impl Extractor {
    fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("py") => Self::Python,
            _ => Self::Unsupported,
        }
    }

    fn extract_imports(&self, source: &str) -> Vec<String> {
        match self {
            Self::Python => extract_python_imports(source),
            Self::Unsupported => vec![],
        }
    }
}

/// Resolve a Python module specifier to candidate local paths.
///
/// Given root `/project` and specifier `a.b.c`, tries:
///   - `a/b/c.py`
///   - `a/b/c/__init__.py`
///   - `a/b.py`          (truncated — handles `from a.b import c`)
///   - `a/b/__init__.py`
///   - `a.py`
fn resolve_python_module(root: &Path, module: &str) -> Vec<PathBuf> {
    let rel: PathBuf = module.replace('.', "/").into();

    let mut candidates = vec![
        root.join(&rel).with_extension("py"),
        root.join(&rel).join("__init__.py"),
    ];

    // Also try truncated forms: `from a.b.c import x` → a/b.py, a.py
    let parts: Vec<&str> = module.split('.').collect();
    for len in (1..parts.len()).rev() {
        let prefix: PathBuf = parts[..len].join("/").into();
        candidates.push(root.join(&prefix).with_extension("py"));
        candidates.push(root.join(&prefix).join("__init__.py"));
    }

    candidates
}

/// Walk forward from `entry_file`, collecting all local files reachable
/// through static imports, bounded to files that live inside `root_dir`.
///
/// Returns paths relative to the current working directory (matching the
/// style used by `collect_files` / `--only`).
pub fn sniff_forward_paths(entry_file: &str, root_dir: &str) -> Vec<String> {
    let root = Path::new(root_dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(root_dir));

    let start = Path::new(entry_file)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(entry_file));

    // Verify the entry file itself lives under root
    if !start.starts_with(&root) {
        eprintln!(
            "⚠️  sniff: entry file '{}' is outside --dir '{}'; aborting.",
            entry_file, root_dir
        );
        return vec![];
    }

    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        if visited.contains(&current) {
            continue;
        }

        // Must exist and be inside root
        if !current.exists() || !current.starts_with(&root) {
            continue;
        }

        visited.insert(current.clone());

        let source = match std::fs::read_to_string(&current) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let extractor = Extractor::from_path(&current);
        let raw_imports = extractor.extract_imports(&source);

        for module in raw_imports {
            let current_dir = current.parent().unwrap_or(&root);

            // Resolve candidates relative to root, then relative to file's dir
            let mut candidates = resolve_python_module(&root, &module);
            candidates.extend(resolve_python_module(current_dir, &module));

            for candidate in candidates {
                if let Ok(canon) = candidate.canonicalize() {
                    if canon.starts_with(&root) && !visited.contains(&canon) {
                        queue.push_back(canon);
                    }
                }
            }
        }
    }

    // Convert canonical absolute paths back to paths that collect_files
    // will recognise.  collect_files walks `args.dir` with WalkDir and
    // produces paths like "../the-graveyard/graveyard/foo.py", i.e.
    // root_dir + "/" + relative-to-root.  We reproduce that here by
    // stripping the canonical root prefix and re-prepending root_dir.
    let root_dir_trimmed = root_dir.trim_end_matches('/');

    let mut result: Vec<String> = visited
        .into_iter()
        .filter_map(|abs| {
            abs.strip_prefix(&root)
                .ok()
                .and_then(|rel| rel.to_str())
                .map(|rel| format!("{}/{}", root_dir_trimmed, rel))
        })
        .collect();

    // Deterministic ordering
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_python_module_simple() {
        let root = Path::new("/project");
        let candidates = resolve_python_module(root, "foo");
        assert!(candidates.contains(&PathBuf::from("/project/foo.py")));
        assert!(candidates.contains(&PathBuf::from("/project/foo/__init__.py")));
    }

    #[test]
    fn test_resolve_python_module_dotted() {
        let root = Path::new("/project");
        let candidates = resolve_python_module(root, "a.b.c");
        assert!(candidates.contains(&PathBuf::from("/project/a/b/c.py")));
        assert!(candidates.contains(&PathBuf::from("/project/a/b/c/__init__.py")));
        // truncated forms
        assert!(candidates.contains(&PathBuf::from("/project/a/b.py")));
        assert!(candidates.contains(&PathBuf::from("/project/a.py")));
    }

    #[test]
    fn test_sniff_empty_on_missing_file() {
        let result = sniff_forward_paths("/nonexistent/file.py", "/nonexistent");
        assert!(result.is_empty());
    }
}