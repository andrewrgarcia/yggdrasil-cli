use std::collections::HashSet;
use std::fs;
use similar::{TextDiff, ChangeTag};

/// Orchestrates full diff across two sets of files
pub fn run_diff(from: Vec<String>, to: Vec<String>) {
    let from_set: HashSet<_> = from.iter().collect();
    let to_set: HashSet<_> = to.iter().collect();

    // Deleted files
    for f in from_set.difference(&to_set) {
        println!("- {}", f);
    }

    // New files
    for f in to_set.difference(&from_set) {
        println!("+ {}", f);
    }

    // Common files → line diffs
    for f in from_set.intersection(&to_set) {
        let from_content = fs::read_to_string(f).unwrap_or_default();
        let to_content = fs::read_to_string(f).unwrap_or_default();
        diff_file_contents(f, &from_content, &to_content);
    }
}

/// Print a line-by-line diff for a single file
pub fn diff_file_contents(file: &str, from_content: &str, to_content: &str) {
    if from_content == to_content {
        return;
    }

    println!("\n📄 Diff for {file}:\n");

    let diff = TextDiff::from_lines(from_content, to_content);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal  => " ",
        };

        // Use ANSI color
        let colored = match change.tag() {
            ChangeTag::Delete => format!("\x1b[91m{}{}\x1b[0m", sign, change),
            ChangeTag::Insert => format!("\x1b[92m{}{}\x1b[0m", sign, change),
            ChangeTag::Equal  => format!(" {}{}", sign, change),
        };
        print!("{}", colored);
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use similar::ChangeTag;

    /// Capture diff output into a string instead of printing to stdout
    fn capture_diff(from: &str, to: &str) -> String {
        let diff = TextDiff::from_lines(from, to);
        let mut out = String::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal  => " ",
            };

            let piece = match change.tag() {
                ChangeTag::Delete => format!("\x1b[91m{}{}\x1b[0m", sign, change),
                ChangeTag::Insert => format!("\x1b[92m{}{}\x1b[0m", sign, change),
                ChangeTag::Equal  => format!(" {}{}", sign, change),
            };
            out.push_str(&piece);
        }

        out
    }

    #[test]
    fn test_diff_identical_strings() {
        let result = capture_diff("hello\nworld\n", "hello\nworld\n");
        assert!(result.contains(" hello"));
        assert!(result.contains(" world"));
        assert!(!result.contains("\x1b[91m-")); // no deletions
        assert!(!result.contains("\x1b[92m+")); // no insertions
    }

    #[test]
    fn test_diff_added_line() {
        let result = capture_diff("hello\n", "hello\nworld\n");
        assert!(result.contains(" hello"));
        assert!(result.contains("\x1b[92m+world\n\x1b[0m")); // added line in green
    }

    #[test]
    fn test_diff_removed_line() {
        let result = capture_diff("hello\nworld\n", "hello\n");
        assert!(result.contains(" hello"));
        assert!(result.contains("\x1b[91m-world\n\x1b[0m")); // removed line in red
    }

    #[test]
    fn test_diff_modified_line() {
        let result = capture_diff("hello\nworld\n", "hello\nmars\n");
        assert!(result.contains(" hello"));
        assert!(result.contains("\x1b[91m-world\n\x1b[0m"));
        assert!(result.contains("\x1b[92m+mars\n\x1b[0m"));
    }
}
